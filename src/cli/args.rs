// src/cli/args.rs
use std::path::PathBuf;
use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    /// Путь к конфигурационному файлу
    #[arg(short, long)]
    pub config: PathBuf,
    
    /// Количество воркеров
    #[arg(short, long)]
    pub workers: Option<usize>,
    
    /// Длительность теста (например: 30s, 5m, 1h)
    #[arg(short, long)]
    pub duration: Option<String>,
    
    /// Путь для сохранения HTML отчёта
    #[arg(short, long)]
    pub report: Option<PathBuf>,
    
    /// Путь для сохранения JSON отчёта
    #[arg(short = 'j', long)]
    pub json: Option<PathBuf>,
    
    /// Режим проверки без выполнения
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct ValidateArgs {
    /// Путь к конфигурационному файлу
    #[arg(short, long)]
    pub config: PathBuf,
    
    /// Показать детальную информацию о конфиге
    #[arg(long, default_value_t = false)]
    pub detail: bool,
}

#[derive(Args)]
pub struct GenerateArgs {
    /// Тип конфига (http, grpc, websocket, multi)
    #[arg(default_value = "http")]
    pub kind: String,
    
    /// Путь для сохранения
    #[arg(short, long, default_value = "config.yaml")]
    pub output: PathBuf,
}