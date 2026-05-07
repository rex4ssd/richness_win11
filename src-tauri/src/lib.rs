// lib.rs — Tauri 2.0 library entry point
// Tauri 2 requires a [lib] crate alongside the binary.
// All Tauri setup lives here; main.rs just calls run().

pub mod commands;
pub mod error;
pub mod hardware;
pub mod logging;
pub mod state;

use commands::*;
use state::AppState;

pub fn run() {
    // Keep log guards alive for the lifetime of the process
    let _log_guards = logging::init_logging();

    tracing::info!("richness-win11 starting");

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // RS-485
            rs485_list_ports,
            rs485_start,
            rs485_stop,
            rs485_send,
            // USB
            usb_list_devices,
            usb_start,
            usb_stop,
            usb_send,
            // RJ45
            rj45_start,
            rj45_stop,
            rj45_send,
        ])
        .run(tauri::generate_context!())
        // ONLY acceptable expect(): Tauri's own event-loop init failure is
        // unrecoverable at the process level — no business logic panics here.
        .expect("error while running tauri application");
}
