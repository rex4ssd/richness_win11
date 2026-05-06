// error.rs — unified error types (thiserror)
// All hardware errors are concrete; commands wrap them into String for IPC.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Rs485Error {
    #[error("Serial port open failed: {0}")]
    Open(#[from] serialport::Error),
    #[error("Serial write error: {0}")]
    Write(#[source] serialport::Error),
    #[error("Already running on port {0}")]
    AlreadyRunning(String),
    #[error("Not started")]
    NotRunning,
}

#[derive(Debug, Error)]
pub enum UsbError {
    #[error("USB error: {0}")]
    Rusb(#[from] rusb::Error),
    #[error("Device not found (VID={vid:#06x} PID={pid:#06x})")]
    NotFound { vid: u16, pid: u16 },
    #[error("Endpoint {0} not found")]
    EndpointNotFound(u8),
    #[error("Already running")]
    AlreadyRunning,
    #[error("Not started")]
    NotRunning,
}

#[derive(Debug, Error)]
pub enum Rj45Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Already running on {0}")]
    AlreadyRunning(String),
    #[error("Not started")]
    NotRunning,
    #[error("Channel send failed")]
    ChannelClosed,
}

// ── Convenience: convert any hardware error to String for Tauri IPC ─────────

pub fn to_ipc<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}
