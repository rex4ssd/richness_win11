// hardware/usb.rs — USB bulk-transfer listener + sender (via rusb)
//
// Design:
//   • Uses rusb's GlobalContext (no per-call Context allocation).
//   • DeviceHandle is NOT Clone → wrapped in Arc<Mutex<…>> so both threads
//     can share the handle safely.
//   • Bulk-IN endpoint: polled continuously in a std::thread (rusb is sync).
//   • Bulk-OUT endpoint: driven by an mpsc channel from callers.
//   • AtomicBool for graceful cancellation.
//
// Windows note: the target USB device must have the WinUSB driver
// installed (use Zadig tool) or rusb returns ACCESS_DENIED.

use crate::error::UsbError;
use crate::hardware::{HwDataEvent, HwLogEvent};
use crate::state::UsbHandle;
use rusb::{DeviceHandle, Direction, GlobalContext, TransferType, UsbContext};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

const BULK_TIMEOUT: Duration = Duration::from_millis(50);
const READ_BUF_SIZE: usize = 65536;

#[derive(Serialize, Clone, Debug)]
pub struct UsbDeviceInfo {
    pub vid: u16,
    pub pid: u16,
    /// "bus-address" string — stable UI identifier
    pub bus_address: String,
    pub description: String,
}

/// Enumerate all USB devices visible to libusb.
pub fn list_devices() -> Result<Vec<UsbDeviceInfo>, UsbError> {
    let mut out = Vec::new();
    for device in rusb::devices()?.iter() {
        let Ok(desc) = device.device_descriptor() else {
            continue;
        };
        out.push(UsbDeviceInfo {
            vid: desc.vendor_id(),
            pid: desc.product_id(),
            bus_address: format!("{}-{}", device.bus_number(), device.address()),
            description: format!(
                "VID={:#06x}  PID={:#06x}  Bus={}  Addr={}",
                desc.vendor_id(),
                desc.product_id(),
                device.bus_number(),
                device.address()
            ),
        });
    }
    Ok(out)
}

/// Open device by VID/PID, discover bulk endpoints, spawn read + write threads.
pub fn start(
    host_id: String,
    vid: u16,
    pid: u16,
    interface_num: u8,
    app: AppHandle,
) -> Result<UsbHandle, UsbError> {
    let handle = rusb::open_device_with_vid_pid(vid, pid)
        .ok_or(UsbError::NotFound { vid, pid })?;

    // Detach kernel driver (no-op on Windows; helpful on Linux)
    handle.set_auto_detach_kernel_driver(true).ok();
    handle.claim_interface(interface_num)?;

    let (bulk_in, bulk_out) = find_bulk_endpoints(&handle, interface_num)?;

    info!(%host_id, vid, pid, bulk_in, bulk_out, "USB device opened");
    emit_log(
        &app,
        HwLogEvent::info(format!(
            "[{host_id}] USB VID={vid:#06x} PID={pid:#06x} if={interface_num}"
        )),
    );

    // Arc<Mutex<…>> lets both threads access the same handle safely.
    // We use std::sync::Mutex here because read_bulk/write_bulk are sync.
    let shared = Arc::new(std::sync::Mutex::new(handle));
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::unbounded_channel::<(u8, Vec<u8>)>();

    spawn_read_thread(
        host_id.clone(),
        shared.clone(),
        bulk_in,
        cancel.clone(),
        app.clone(),
    );
    spawn_write_thread(
        host_id.clone(),
        shared,
        bulk_out,
        rx,
        cancel.clone(),
        app.clone(),
    );

    Ok(UsbHandle { cancel, tx })
}

// ── Endpoint discovery ────────────────────────────────────────────────────────

fn find_bulk_endpoints(
    handle: &DeviceHandle<GlobalContext>,
    interface_num: u8,
) -> Result<(u8, u8), UsbError> {
    let device = handle.device();
    let config = device.active_config_descriptor()?;

    for iface in config.interfaces() {
        if iface.number() != interface_num {
            continue;
        }
        for desc in iface.descriptors() {
            let mut bulk_in = None;
            let mut bulk_out = None;
            for ep in desc.endpoint_descriptors() {
                if ep.transfer_type() == TransferType::Bulk {
                    match ep.direction() {
                        Direction::In => bulk_in = Some(ep.address()),
                        Direction::Out => bulk_out = Some(ep.address()),
                    }
                }
            }
            if let (Some(i), Some(o)) = (bulk_in, bulk_out) {
                return Ok((i, o));
            }
        }
    }
    Err(UsbError::EndpointNotFound(interface_num))
}

// ── Background threads ────────────────────────────────────────────────────────

fn spawn_read_thread(
    host_id: String,
    shared: Arc<std::sync::Mutex<DeviceHandle<GlobalContext>>>,
    endpoint: u8,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) {
    std::thread::spawn(move || {
        let mut buf = vec![0u8; READ_BUF_SIZE];
        info!(%host_id, endpoint, "USB read thread started");

        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            let result = {
                // Lock only for the duration of the call; don't hold across sleep
                match shared.lock() {
                    Ok(h) => h.read_bulk(endpoint, &mut buf, BULK_TIMEOUT),
                    Err(_) => break, // poisoned mutex — shouldn't happen
                }
            };

            match result {
                Ok(n) if n > 0 => {
                    let payload = HwDataEvent::new(&host_id, "usb", &buf[..n]);
                    if let Err(e) = app.emit("usb-data", &payload) {
                        error!(%host_id, "emit usb-data: {e}");
                    }
                }
                Ok(_) => {}
                Err(rusb::Error::Timeout) => {}
                Err(e) => {
                    let msg = format!("[{host_id}] USB read error: {e}");
                    error!("{msg}");
                    emit_log(&app, HwLogEvent::error(&msg));
                    break;
                }
            }
        }
        info!(%host_id, "USB read thread stopped");
    });
}

fn spawn_write_thread(
    host_id: String,
    shared: Arc<std::sync::Mutex<DeviceHandle<GlobalContext>>>,
    endpoint: u8,
    mut rx: mpsc::UnboundedReceiver<(u8, Vec<u8>)>,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) {
    tokio::spawn(async move {
        while let Some((ep, data)) = rx.recv().await {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let target_ep = if ep == 0 { endpoint } else { ep };
            let result = match shared.lock() {
                Ok(h) => h.write_bulk(target_ep, &data, BULK_TIMEOUT),
                Err(_) => break,
            };
            match result {
                Ok(n) => info!(%host_id, bytes = n, "USB sent"),
                Err(e) => {
                    let msg = format!("[{host_id}] USB write error: {e}");
                    error!("{msg}");
                    emit_log(&app, HwLogEvent::error(&msg));
                }
            }
        }
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn emit_log(app: &AppHandle, event: HwLogEvent) {
    if let Err(e) = app.emit("hw-log", &event) {
        warn!("emit hw-log: {e}");
    }
}
