// src/protocols/grpc/client.rs
use crate::errors::{BombardierError, BombardierResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tonic::transport::Channel;
use tonic::Request;
use tracing::debug;

pub struct GrpcExecutor {
    url: String,
}

impl GrpcExecutor {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn execute(
        &self,
        request_data: &HashMap<String, String>,
        timeout: Option<Duration>,
    ) -> BombardierResult<GrpcResponse> {
        let start = Instant::now();
        
        // Создаём канал
        let channel = Channel::from_shared(self.url.clone())
            .map_err(|e| BombardierError::Grpc(format!("Ошибка создания канала: {}", e)))?
            .connect()
            .await
            .map_err(|e| BombardierError::Grpc(format!("Ошибка подключения: {}", e)))?;
        
        // Устанавливаем таймаут
        let timeout_dur = timeout.unwrap_or(Duration::from_secs(30));
        
        // Получаем метод из запроса
        let method = request_data
            .get("method")
            .map(|s| s.as_str())
            .unwrap_or("SayHello");
        
        // Создаём клиент и вызываем нужный метод
        let response = match method {
            "SayHello" => {
                let mut client = super::helloworld::greeter_client::GreeterClient::new(channel);
                let name = request_data.get("name").unwrap_or(&"World".to_string()).clone();
                let request = Request::new(super::helloworld::HelloRequest { name });
                
                let resp = tokio::time::timeout(timeout_dur, client.say_hello(request))
                    .await
                    .map_err(|_| BombardierError::Timeout(
                        format!("Таймаут gRPC запроса к {}", self.url)
                    ))?
                    .map_err(|e| BombardierError::Grpc(format!("Ошибка gRPC: {}", e)))?;
                
                resp.into_inner()
            }
            // Для grpcbin используем другой метод
            _ => {
                // Пробуем вызвать grpcbin.GRPCBin/UnaryCall
                // Для простоты пока возвращаем ошибку
                return Err(BombardierError::Grpc(
                    format!("Метод '{}' не поддерживается", method)
                ));
            }
        };
        
        let elapsed = start.elapsed();
        
        debug!("gRPC ответ: {}ms", elapsed.as_millis());
        
        Ok(GrpcResponse {
            message: response.message,
            latency: elapsed,
            success: true,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GrpcResponse {
    pub message: String,
    pub latency: Duration,
    pub success: bool,
}