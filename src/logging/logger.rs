// src/logging/logger.rs
use tracing_subscriber::{fmt, EnvFilter};

pub fn init(level: String) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_line_number(true)
        .with_file(true)
        .init();
}