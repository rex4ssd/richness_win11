// logging.rs — initialise tracing: console + rolling file under AppData/logs
//
// File rotation: daily.  Format: YYYY-MM-DD.log
// On non-Windows dev machines the log dir falls back to the current directory.

use std::path::PathBuf;
use tracing::Level;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging() -> Vec<non_blocking::WorkerGuard> {
    let log_dir = resolve_log_dir();
    std::fs::create_dir_all(&log_dir).ok();

    // Rolling file appender (daily rotation)
    let file_appender = rolling::daily(&log_dir, "richness");
    let (file_writer, file_guard) = non_blocking(file_appender);

    // Console writer
    let (console_writer, console_guard) = non_blocking(std::io::stdout());

    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false)
                .with_timer(ChronoLocal::rfc_3339()),
        )
        .with(
            fmt::layer()
                .with_writer(console_writer)
                .with_ansi(true)
                .with_timer(ChronoLocal::rfc_3339()),
        )
        .init();

    // Return guards so they live for the lifetime of main()
    vec![file_guard, console_guard]
}

fn resolve_log_dir() -> PathBuf {
    // Windows: %APPDATA%\richness-win11\logs
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata)
                .join("richness-win11")
                .join("logs");
        }
    }

    // macOS / Linux (dev): ./logs/
    PathBuf::from("logs")
}
