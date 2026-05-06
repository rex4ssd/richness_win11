// commands/rs485_cmd.rs — Tauri IPC commands for RS-485

use crate::error::to_ipc;
use crate::hardware::rs485;
use crate::state::AppState;
use tauri::{AppHandle, State};
use tracing::{info, warn};

/// Return list of available serial port names.
#[tauri::command]
pub async fn rs485_list_ports() -> Result<Vec<String>, String> {
    rs485::list_ports().map_err(to_ipc)
}

/// Open port and start background listener.
#[tauri::command]
pub async fn rs485_start(
    host_id: String,
    port: String,
    baud: u32,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut hosts = state.hosts.lock().await;
    let host = hosts.entry(host_id.clone()).or_default();

    if host.rs485.is_some() {
        return Err(format!("[{host_id}] RS-485 already running"));
    }

    let handle = rs485::start(host_id.clone(), port, baud, app).map_err(to_ipc)?;
    host.rs485 = Some(handle);
    info!(%host_id, "rs485_start OK");
    Ok(())
}

/// Stop background listener and close the port.
#[tauri::command]
pub async fn rs485_stop(host_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut hosts = state.hosts.lock().await;
    let host = hosts.entry(host_id.clone()).or_default();

    match host.rs485.take() {
        Some(h) => {
            h.cancel();
            info!(%host_id, "rs485_stop OK");
            Ok(())
        }
        None => {
            warn!(%host_id, "rs485_stop: not running");
            Err(format!("[{host_id}] RS-485 not running"))
        }
    }
}

/// Send raw bytes through the open serial port.
#[tauri::command]
pub async fn rs485_send(
    host_id: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let hosts = state.hosts.lock().await;
    let host = hosts
        .get(&host_id)
        .ok_or_else(|| format!("[{host_id}] RS-485 not started"))?;
    let handle = host
        .rs485
        .as_ref()
        .ok_or_else(|| format!("[{host_id}] RS-485 not started"))?;

    handle
        .tx
        .send(data)
        .map_err(|_| format!("[{host_id}] RS-485 channel closed"))
}
