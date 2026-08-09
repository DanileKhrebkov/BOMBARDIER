// src/executor/worker.rs
use crate::config::Step;
use crate::errors::BombardierResult;
use crate::protocols::{HttpExecutor, WebSocketExecutor, WebSocketStep};
use super::context::Context;
use tracing::debug;

pub struct Worker {
    pub id: usize,
    http_executor: HttpExecutor,
    websocket_executor: WebSocketExecutor,
}

impl Worker {
    pub fn new(id: usize) -> BombardierResult<Self> {
        Ok(Self {
            id,
            http_executor: HttpExecutor::new()?,
            websocket_executor: WebSocketExecutor::new()?,
        })
    }

    pub async fn execute_step(&self, step: &Step, context: &Context) -> BombardierResult<()> {
        let context_vars = context.get_all();

        match step.protocol {
            crate::config::Protocol::Http => {
                debug!("Воркер {} выполняет HTTP шаг: {}", self.id, step.name);
                let response = self.http_executor.execute(step, &context_vars).await?;
                
                // Извлекаем переменные
                for extract in &step.extract {
                    let value = if let Some(jsonpath) = &extract.jsonpath {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response.body_str) {
                            let mut selector = jsonpath_lib::selector(&json);
                            if let Ok(result) = selector(jsonpath) {
                                if let Some(first) = result.first() {
                                    Some(first.to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else if let Some(regex) = &extract.regex {
                        if let Ok(re) = regex::Regex::new(regex) {
                            if let Some(captures) = re.captures(&response.body_str) {
                                captures.get(1).map(|m| m.as_str().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    if let Some(value) = value {
                        context.set(&extract.name, value);
                    }
                }
            }
            crate::config::Protocol::WebSocket => {
                debug!("Воркер {} выполняет WebSocket шаг: {}", self.id, step.name);
                
                let ws_step = WebSocketStep {
                    name: step.name.clone(),
                    url: step.url.clone(),
                    headers: step.headers.clone(),
                    messages: step.messages.clone(),
                    extract: vec![],
                    timeout: step.timeout,
                    think_time: step.think_time,
                };
                
                let response = self.websocket_executor.execute(&ws_step, &context_vars).await?;
                
                if !response.success {
                    return Err(crate::errors::BombardierError::WebSocket(
                        response.error.unwrap_or_else(|| "Неизвестная ошибка WebSocket".to_string())
                    ));
                }
                
                debug!("Воркер {} WebSocket завершён: {} сообщений, {}ms", 
                    self.id, response.messages.len(), response.latency.as_millis());
            }
            crate::config::Protocol::Grpc => {
                return Err(crate::errors::BombardierError::Grpc(
                    "gRPC пока не поддерживается".to_string()
                ));
            }
        }

        // Think time
        if let Some(think_time) = step.think_time {
            debug!("Воркер {} ожидает {}ms", self.id, think_time.as_millis());
            tokio::time::sleep(think_time).await;
        }

        Ok(())
    }
}