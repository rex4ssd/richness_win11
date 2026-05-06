// hardware/rj45.rs — RJ45 TCP/UDP communication
//
// Supported modes (passed as a string from the frontend):
//   "tcp_client" — connect to remote addr (host:port)
//   "tcp_server" — listen on addr (0.0.0.0:port)
//   "udp"        — bind to local addr, send/recv to remote addr
//
// All received bytes are emitted as "rj45-data" Tauri events.
// An mpsc channel lets callers send bytes without blocking.

use crate::error::Rj45Error;
use crate::hardware::{HwDataEvent, HwLogEvent};
use crate::state::Rj45Handle;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::mpsc,
};
use tracing::{error, info, warn};

const READ_BUF_SIZE: usize = 65536;

pub async fn start(
    host_id: String,
    mode: String,        // "tcp_client" | "tcp_server" | "udp"
    addr: String,        // e.g. "192.168.1.10:8080" or "0.0.0.0:8080"
    remote_addr: String, // for UDP send target (ignored for tcp_server)
    app: AppHandle,
) -> Result<Rj45Handle, Rj45Error> {
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::unbounded_channel::<Vec<u8>>();

    info!(%host_id, %mode, %addr, "RJ45 start");
    emit_log(
        &app,
        HwLogEvent::info(format!("[{host_id}] RJ45 {mode} → {addr}")),
    );

    match mode.as_str() {
        "tcp_client" => {
            spawn_tcp_client(host_id, addr, rx, cancel.clone(), app).await?;
        }
        "tcp_server" => {
            spawn_tcp_server(host_id, addr, rx, cancel.clone(), app).await?;
        }
        "udp" => {
            spawn_udp(host_id, addr, remote_addr, rx, cancel.clone(), app).await?;
        }
        other => {
            return Err(Rj45Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown RJ45 mode: {other}"),
            )))
        }
    }

    Ok(Rj45Handle { cancel, tx })
}

// ── TCP Client ────────────────────────────────────────────────────────────────

async fn spawn_tcp_client(
    host_id: String,
    addr: String,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) -> Result<(), Rj45Error> {
    let stream = TcpStream::connect(&addr).await?;
    info!(%host_id, %addr, "TCP client connected");
    emit_log(&app, HwLogEvent::info(format!("[{host_id}] TCP connected to {addr}")));

    tokio::spawn(async move {
        let (mut reader, mut writer) = stream.into_split();
        let mut buf = vec![0u8; READ_BUF_SIZE];

        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            tokio::select! {
                // Receive from remote
                result = reader.read(&mut buf) => {
                    match result {
                        Ok(0) => {
                            let msg = format!("[{host_id}] TCP connection closed by peer");
                            warn!("{msg}");
                            emit_log(&app, HwLogEvent::warn(&msg));
                            break;
                        }
                        Ok(n) => {
                            let payload = HwDataEvent::new(&host_id, "rj45", &buf[..n]);
                            if let Err(e) = app.emit("rj45-data", &payload) {
                                error!(%host_id, "emit rj45-data: {e}");
                            }
                        }
                        Err(e) => {
                            let msg = format!("[{host_id}] TCP read error: {e}");
                            error!("{msg}");
                            emit_log(&app, HwLogEvent::error(&msg));
                            break;
                        }
                    }
                }
                // Send to remote
                Some(data) = rx.recv() => {
                    if let Err(e) = writer.write_all(&data).await {
                        let msg = format!("[{host_id}] TCP write error: {e}");
                        error!("{msg}");
                        emit_log(&app, HwLogEvent::error(&msg));
                        break;
                    }
                    info!(%host_id, bytes = data.len(), "TCP sent");
                }
            }
        }
        info!(%host_id, "TCP client task stopped");
    });

    Ok(())
}

// ── TCP Server ────────────────────────────────────────────────────────────────

async fn spawn_tcp_server(
    host_id: String,
    addr: String,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) -> Result<(), Rj45Error> {
    let listener = TcpListener::bind(&addr).await?;
    info!(%host_id, %addr, "TCP server listening");
    emit_log(&app, HwLogEvent::info(format!("[{host_id}] TCP server on {addr}")));

    tokio::spawn(async move {
        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            match listener.accept().await {
                Ok((stream, peer)) => {
                    info!(%host_id, %peer, "TCP server: client connected");
                    emit_log(&app, HwLogEvent::info(format!("[{host_id}] TCP client {peer}")));

                    // Each accepted connection gets its own sub-task.
                    let h2 = host_id.clone();
                    let a2 = app.clone();
                    let c2 = cancel.clone();
                    // Forward channel messages to this connection (simple: last connected wins)
                    // For production N-client: extend to BroadcastSender
                    let (mut reader, mut writer) = stream.into_split();
                    let mut buf = vec![0u8; READ_BUF_SIZE];

                    tokio::spawn(async move {
                        loop {
                            if c2.load(Ordering::Relaxed) { break; }
                            match reader.read(&mut buf).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    let payload = HwDataEvent::new(&h2, "rj45", &buf[..n]);
                                    if let Err(e) = a2.emit("rj45-data", &payload) {
                                        error!(%h2, "emit rj45-data: {e}");
                                    }
                                }
                                Err(e) => {
                                    error!(%h2, "TCP server read: {e}");
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!(%host_id, "TCP accept error: {e}");
                    break;
                }
            }
        }
        drop(rx); // drain channel
        info!(%host_id, "TCP server task stopped");
    });

    Ok(())
}

// ── UDP ───────────────────────────────────────────────────────────────────────

async fn spawn_udp(
    host_id: String,
    bind_addr: String,
    remote_addr: String,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) -> Result<(), Rj45Error> {
    let socket = UdpSocket::bind(&bind_addr).await?;
    if !remote_addr.is_empty() {
        socket.connect(&remote_addr).await?;
    }
    info!(%host_id, %bind_addr, %remote_addr, "UDP socket ready");
    emit_log(&app, HwLogEvent::info(format!("[{host_id}] UDP bound {bind_addr}")));

    tokio::spawn(async move {
        let mut buf = vec![0u8; READ_BUF_SIZE];

        loop {
            if cancel.load(Ordering::Relaxed) { break; }

            tokio::select! {
                result = socket.recv(&mut buf) => {
                    match result {
                        Ok(n) => {
                            let payload = HwDataEvent::new(&host_id, "rj45", &buf[..n]);
                            if let Err(e) = app.emit("rj45-data", &payload) {
                                error!(%host_id, "emit rj45-data: {e}");
                            }
                        }
                        Err(e) => {
                            let msg = format!("[{host_id}] UDP recv error: {e}");
                            error!("{msg}");
                            emit_log(&app, HwLogEvent::error(&msg));
                            break;
                        }
                    }
                }
                Some(data) = rx.recv() => {
                    if let Err(e) = socket.send(&data).await {
                        let msg = format!("[{host_id}] UDP send error: {e}");
                        error!("{msg}");
                        emit_log(&app, HwLogEvent::error(&msg));
                    } else {
                        info!(%host_id, bytes = data.len(), "UDP sent");
                    }
                }
            }
        }
        info!(%host_id, "UDP task stopped");
    });

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn emit_log(app: &AppHandle, event: HwLogEvent) {
    if let Err(e) = app.emit("hw-log", &event) {
        warn!("emit hw-log: {e}");
    }
}
