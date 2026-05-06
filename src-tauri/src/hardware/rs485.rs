// hardware/rs485.rs — RS-485 / serial-port listener + sender
//
// Design:
//   • open() is called once; a std::thread runs the read loop (serialport is sync).
//   • An mpsc::UnboundedSender<Vec<u8>> lets callers write data asynchronously.
//   • An AtomicBool signals the background thread to exit cleanly.
//   • All received bytes are emitted as "rs485-data" Tauri events.

use crate::error::Rs485Error;
use crate::hardware::{HwDataEvent, HwLogEvent};
use crate::state::Rs485Handle;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

const READ_TIMEOUT_MS: u64 = 50;
const READ_BUF_SIZE: usize = 4096;

/// List available serial port names.
pub fn list_ports() -> Result<Vec<String>, Rs485Error> {
    let ports = serialport::available_ports()?;
    Ok(ports.into_iter().map(|p| p.port_name).collect())
}

/// Open a serial port and spawn background read + write threads.
/// Returns an Rs485Handle that can be stored in AppState.
pub fn start(
    host_id: String,
    port_name: String,
    baud_rate: u32,
    app: AppHandle,
) -> Result<Rs485Handle, Rs485Error> {
    let port = serialport::new(&port_name, baud_rate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(READ_TIMEOUT_MS))
        .open()
        .map_err(Rs485Error::Open)?;

    info!(%host_id, %port_name, %baud_rate, "RS-485 port opened");
    emit_log(&app, HwLogEvent::info(format!("[{host_id}] RS-485 opened {port_name}@{baud_rate}")));

    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Clone port for the write thread
    let write_port = port.try_clone().map_err(Rs485Error::Open)?;

    spawn_read_thread(host_id.clone(), port, cancel.clone(), app.clone());
    spawn_write_thread(host_id.clone(), write_port, rx, cancel.clone(), app.clone());

    Ok(Rs485Handle { cancel, tx })
}

// ── Background threads ────────────────────────────────────────────────────────

fn spawn_read_thread(
    host_id: String,
    mut port: Box<dyn SerialPort>,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) {
    std::thread::spawn(move || {
        let mut buf = [0u8; READ_BUF_SIZE];
        info!(%host_id, "RS-485 read thread started");

        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let payload = HwDataEvent::new(&host_id, "rs485", &buf[..n]);
                    if let Err(e) = app.emit("rs485-data", &payload) {
                        error!(%host_id, "emit rs485-data: {e}");
                    }
                }
                Ok(_) => {} // timeout, no data — loop
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    let msg = format!("[{host_id}] RS-485 read error: {e}");
                    error!("{msg}");
                    emit_log(&app, HwLogEvent::error(&msg));
                    break;
                }
            }
        }
        info!(%host_id, "RS-485 read thread stopped");
    });
}

fn spawn_write_thread(
    host_id: String,
    mut port: Box<dyn SerialPort>,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) {
    // Run in a blocking tokio task so the channel is drained async-safely.
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            if let Err(e) = port.write_all(&data) {
                let msg = format!("[{host_id}] RS-485 write error: {e}");
                error!("{msg}");
                emit_log(&app, HwLogEvent::error(&msg));
            } else {
                info!(%host_id, bytes = data.len(), "RS-485 sent");
            }
        }
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn emit_log(app: &AppHandle, event: HwLogEvent) {
    if let Err(e) = app.emit("hw-log", &event) {
        warn!("emit hw-log failed: {e}");
    }
}
