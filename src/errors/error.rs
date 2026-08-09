use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum BombardierError {
    #[error("Ошибка конфигурации: {0}")]
    Config(String),
    
    #[error("Ошибка парсинга YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Ошибка парсинга JSON: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Ошибка HTTP: {0}")]
    Http(String),
    
    #[error("Ошибка gRPC: {0}")]
    Grpc(String),
    
    #[error("Ошибка WebSocket: {0}")]
    WebSocket(String),
    
    #[error("IO ошибка: {0}")]
    Io(#[from] io::Error),
    
    #[error("Таймаут выполнения: {0}")]
    Timeout(String),
    
    #[error("Ошибка валидации: {0}")]
    Validation(String),
    
    #[error("Ошибка экстракции: {0}")]
    Extraction(String),
    
    #[error("Ошибка генерации данных: {0}")]
    Generator(String),
    
    #[error("Ошибка метрик: {0}")]
    Metrics(String),
    
    #[error("Ошибка отчёта: {0}")]
    Report(String),
    
    #[error("Ошибка ассерта: {0}")]
    Assertion(String),
    
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

pub type BombardierResult<T> = Result<T, BombardierError>;