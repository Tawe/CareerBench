//! Logging utilities for error tracking and debugging

use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::fs::File;
use crate::db::get_app_data_dir;

static LOG_INITIALIZED: std::sync::Once = std::sync::Once::new();

/// Initialize logging to file
pub fn init_logging() {
    LOG_INITIALIZED.call_once(|| {
        let log_path = get_app_data_dir().join("careerbench.log");
        
        // Try to open log file (append mode)
        match File::options().create(true).append(true).open(&log_path) {
            Ok(log_file) => {
                let config = Config::default();
                
                // Use default config - simplelog will handle formatting
                let config = Config::default();
                
                if let Err(e) = CombinedLogger::init(vec![
                    WriteLogger::new(
                        LevelFilter::Debug, // Log everything at debug level and above
                        config,
                        log_file,
                    ),
                ]) {
                    eprintln!("Failed to initialize logger: {}", e);
                } else {
                    log::info!("=== CareerBench Logging Initialized ===");
                    log::info!("Log file: {}", log_path.display());
                }
            }
            Err(e) => {
                eprintln!("Failed to open log file at {}: {}", log_path.display(), e);
            }
        }
    });
}

/// Log a panic with full backtrace
pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_message = panic_info.payload().downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown panic".to_string());
        
        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        let message = format!(
            "PANIC: {}\nLocation: {}\nTime: {}",
            panic_message,
            location,
            chrono::Utc::now().to_rfc3339()
        );
        
        // Try to log if logger is initialized
        log::error!("{}", message);
        eprintln!("{}", message);
        
        // Also write to a crash log file
        if let Ok(mut crash_log) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(get_app_data_dir().join("crash.log"))
        {
            use std::io::Write;
            let _ = writeln!(crash_log, "{}", message);
        }
    }));
}

/// Log an error with context
pub fn log_error(context: &str, error: &dyn std::error::Error) {
    log::error!("[{}] Error: {}", context, error);
    if let Some(source) = error.source() {
        log::error!("[{}] Caused by: {}", context, source);
    }
}

/// Log a warning with context
pub fn log_warning(context: &str, message: &str) {
    log::warn!("[{}] {}", context, message);
}

/// Log debug information
pub fn log_debug(context: &str, message: &str) {
    log::debug!("[{}] {}", context, message);
}

