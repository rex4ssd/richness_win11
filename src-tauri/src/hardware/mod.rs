// hardware/mod.rs — shared event payload types + sub-module re-exports

pub mod rs485;
pub mod rj45;
pub mod usb;

use chrono::Local;
use serde::Serialize;

/// Generic hardware data event — emitted to frontend via Tauri IPC.
#[derive(Serialize, Clone, Debug)]
pub struct HwDataEvent {
    /// Which host sent this (e.g. "local", "192.168.1.5")
    pub host_id: String,
    /// "rs485" | "usb" | "rj45"
    pub interface: String,
    /// ISO-8601 timestamp
    pub timestamp: String,
    /// Hex dump of raw bytes (uppercase, space-separated)
    pub hex: String,
    /// Length of raw payload in bytes
    pub raw_len: usize,
}

impl HwDataEvent {
    pub fn new(host_id: &str, interface: &str, data: &[u8]) -> Self {
        Self {
            host_id: host_id.to_owned(),
            interface: interface.to_owned(),
            timestamp: Local::now().to_rfc3339(),
            hex: data
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" "),
            raw_len: data.len(),
        }
    }
}

/// Log-level event pushed to frontend alongside tracing output.
#[derive(Serialize, Clone, Debug)]
pub struct HwLogEvent {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

impl HwLogEvent {
    pub fn info(msg: impl Into<String>) -> Self {
        Self {
            level: "INFO".into(),
            message: msg.into(),
            timestamp: Local::now().to_rfc3339(),
        }
    }
    pub fn warn(msg: impl Into<String>) -> Self {
        Self {
            level: "WARN".into(),
            message: msg.into(),
            timestamp: Local::now().to_rfc3339(),
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            level: "ERROR".into(),
            message: msg.into(),
            timestamp: Local::now().to_rfc3339(),
        }
    }
}
