// commands/rj45_cmd.rs — Tauri IPC commands for RJ45 (TCP/UDP)

use crate::error::to_ipc;
use crate::hardware::rj45;
use crate::state::AppState;
use tauri::{AppHandle, State};
use tracing::{info, warn};

/// Start RJ45 connection.
/// mode: "tcp_client" | "tcp_server" | "udp"
/// addr: "host:port" for client/udp, "0.0.0.0:port" for server/udp bind
/// remote_addr: only relevant for UDP (send target); ignored otherwise
#[tauri::command]
pub async fn rj45_start(
    host_id: String,
    mode: String,
    addr: String,
    remote_addr: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut hosts = state.hosts.lock().await;
    let host = hosts.entry(host_id.clone()).or_default();

    if host.rj45.is_some() {
        return Err(format!("[{host_id}] RJ45 already running"));
    }

    let handle = rj45::start(host_id.clone(), mode, addr, remote_addr, app)
        .await
        .map_err(to_ipc)?;

    host.rj45 = Some(handle);
    info!(%host_id, "rj45_start OK");
    Ok(())
}

/// Stop RJ45 listener / connection.
#[tauri::command]
pub async fn rj45_stop(host_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut hosts = state.hosts.lock().await;
    let host = hosts.entry(host_id.clone()).or_default();

    match host.rj45.take() {
        Some(h) => {
            h.cancel();
            info!(%host_id, "rj45_stop OK");
            Ok(())
        }
        None => {
            warn!(%host_id, "rj45_stop: not running");
            Err(format!("[{host_id}] RJ45 not running"))
        }
    }
}

/// Send bytes over the active RJ45 connection.
#[tauri::command]
pub async fn rj45_send(
    host_id: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let hosts = state.hosts.lock().await;
    let host = hosts
        .get(&host_id)
        .ok_or_else(|| format!("[{host_id}] RJ45 not started"))?;
    let handle = host
        .rj45
        .as_ref()
        .ok_or_else(|| format!("[{host_id}] RJ45 not started"))?;

    handle
        .tx
        .send(data)
        .map_err(|_| format!("[{host_id}] RJ45 channel closed"))
}
