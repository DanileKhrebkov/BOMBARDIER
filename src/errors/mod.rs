// src/errors/mod.rs
mod error;
mod result;

// Не используем glob импорты, чтобы избежать конфликтов
pub use error::BombardierError;
pub use result::BombardierResult;