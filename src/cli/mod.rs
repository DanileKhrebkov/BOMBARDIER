// src/cli/mod.rs
mod commands;
mod args;

pub use commands::Commands;
pub use args::{RunArgs, ValidateArgs, GenerateArgs};

use clap::Parser;

#[derive(Parser)]
#[command(name = "bombardier")]
#[command(about = "CLI-инструмент для нагрузочного тестирования")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Уровень логирования (debug, info, warn, error, trace)
    #[arg(short, long, global = true, default_value = "info")]
    pub verbose: String,
}

// Убираем дублирующий verbose в ValidateArgs