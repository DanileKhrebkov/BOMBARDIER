// src/lib.rs
// Экспортируем все публичные модули
pub mod cli;
pub mod config;
pub mod executor;
pub mod protocols;
pub mod generator;
pub mod metrics;
pub mod reporters;
pub mod assertions;
pub mod errors;
pub mod logging;
pub mod utils;

// Публичный API
pub use errors::{BombardierError, BombardierResult};