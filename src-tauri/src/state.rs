// state.rs — global AppState
//
// Designed for N-host scalability: each host has its own handles keyed by
// host_id (e.g., "local", "192.168.1.10:5000").
// Hosts are lazily inserted on first Start command.

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{mpsc, Mutex};

// ── Per-interface handles ─────────────────────────────────────────────────────

pub struct Rs485Handle {
    /// Set to true to signal background thread to stop.
    pub cancel: Arc<AtomicBool>,
    /// Sender side: caller writes bytes, background thread forwards to port.
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl Rs485Handle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

pub struct UsbHandle {
    pub cancel: Arc<AtomicBool>,
    pub tx: mpsc::UnboundedSender<(u8, Vec<u8>)>, // (endpoint, data)
}

impl UsbHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

pub struct Rj45Handle {
    pub cancel: Arc<AtomicBool>,
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl Rj45Handle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

// ── Per-host state ────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct HostState {
    pub rs485: Option<Rs485Handle>,
    pub usb: Option<UsbHandle>,
    pub rj45: Option<Rj45Handle>,
}

// ── Global application state ──────────────────────────────────────────────────

pub struct AppState {
    /// key = host_id string
    pub hosts: Mutex<HashMap<String, HostState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            hosts: Mutex::new(HashMap::new()),
        }
    }
}
