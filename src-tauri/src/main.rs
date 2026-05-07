// main.rs — binary entry point (Tauri 2 convention)
// All logic is in lib.rs; this file only calls run().

// Prevent a console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    richness_win11_lib::run();
}
