// src/cli/commands.rs
use super::args::{GenerateArgs, RunArgs, ValidateArgs};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Запуск нагрузочного теста
    Run(RunArgs),
    
    /// Проверка конфигурации без выполнения
    Validate(ValidateArgs),
    
    /// Генерация примера конфигурации
    Generate(GenerateArgs),
}