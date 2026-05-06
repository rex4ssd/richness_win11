// main.rs — Tauri 2.0 entry point
// Initialises logging, registers global state, wires up all IPC commands.

// Prevent a console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod hardware;
mod logging;
mod state;

use commands::*;
use state::AppState;

fn main() {
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
