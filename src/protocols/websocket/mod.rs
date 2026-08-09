// src/protocols/websocket/mod.rs
mod client;
mod message;

pub use client::WebSocketExecutor;
pub use client::WebSocketResponse;
pub use message::{WebSocketMessage, WebSocketStep};