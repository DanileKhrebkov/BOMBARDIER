// src/protocols/grpc/mod.rs
mod client;

pub use client::{GrpcExecutor, GrpcResponse};

// Включаем сгенерированный код из build.rs
pub mod helloworld {
    tonic::include_proto!("helloworld");
}