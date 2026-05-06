// commands/usb_cmd.rs — Tauri IPC commands for USB

use crate::error::to_ipc;
use crate::hardware::usb::{self, UsbDeviceInfo};
use crate::state::AppState;
use tauri::{AppHandle, State};
use tracing::{info, warn};

/// Enumerate connected USB devices.
#[tauri::command]
pub async fn usb_list_devices() -> Result<Vec<UsbDeviceInfo>, String> {
    // rusb is sync; offload to blocking thread pool to avoid blocking the async executor.
    tokio::task::spawn_blocking(usb::list_devices)
        .await
        .map_err(|e| e.to_string())?
        .map_err(to_ipc)
}

/// Open USB device and start bulk-IN listener.
#[tauri::command]
pub async fn usb_start(
    host_id: String,
    vid: u16,
    pid: u16,
    interface_num: u8,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut hosts = state.hosts.lock().await;
    let host = hosts.entry(host_id.clone()).or_default();

    if host.usb.is_some() {
        return Err(format!("[{host_id}] USB already running"));
    }

    // usb::start is sync-heavy internally; keep it on blocking pool
    let h_id = host_id.clone();
    let handle = tokio::task::spawn_blocking(move || usb::start(h_id, vid, pid, interface_num, app))
        .await
        .map_err(|e| e.to_string())?
        .map_err(to_ipc)?;

    host.usb = Some(handle);
    info!(%host_id, "usb_start OK");
    Ok(())
}

/// Stop USB listener.
#[tauri::command]
pub async fn usb_stop(host_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut hosts = state.hosts.lock().await;
    let host = hosts.entry(host_id.clone()).or_default();

    match host.usb.take() {
        Some(h) => {
            h.cancel();
            info!(%host_id, "usb_stop OK");
            Ok(())
        }
        None => {
            warn!(%host_id, "usb_stop: not running");
            Err(format!("[{host_id}] USB not running"))
        }
    }
}

/// Send bulk-OUT data. endpoint=0 means use discovered OUT endpoint.
#[tauri::command]
pub async fn usb_send(
    host_id: String,
    endpoint: u8,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let hosts = state.hosts.lock().await;
    let host = hosts
        .get(&host_id)
        .ok_or_else(|| format!("[{host_id}] USB not started"))?;
    let handle = host
        .usb
        .as_ref()
        .ok_or_else(|| format!("[{host_id}] USB not started"))?;

    handle
        .tx
        .send((endpoint, data))
        .map_err(|_| format!("[{host_id}] USB channel closed"))
}
