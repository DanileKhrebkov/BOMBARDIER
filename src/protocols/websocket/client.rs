// src/protocols/websocket/client.rs
use crate::errors::{BombardierError, BombardierResult};
use crate::protocols::websocket::message::WebSocketStep;  // Убираем WebSocketMessage
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::debug;


pub struct WebSocketExecutor;

impl WebSocketExecutor {
    pub fn new() -> BombardierResult<Self> {
        Ok(Self)
    }

    pub async fn execute(
        &self,
        step: &WebSocketStep,
        context: &HashMap<String, String>,
    ) -> BombardierResult<WebSocketResponse> {
        let start = Instant::now();
        
        // Подставляем переменные в URL
        let url = self.render_template(&step.url, context);
        debug!("Подключаемся к WebSocket: {}", url);
        
        // Устанавливаем таймаут на подключение
        let timeout = step.timeout.unwrap_or(Duration::from_secs(30));
        
        // Используем простой URI для подключения
        let connect_future = connect_async(&url);
        
        let (ws_stream, _) = tokio::time::timeout(timeout, connect_future)
            .await
            .map_err(|_| BombardierError::Timeout(
                format!("Таймаут подключения к WebSocket: {}", step.url)
            ))?
            .map_err(|e| BombardierError::WebSocket(format!("Ошибка подключения: {}", e)))?;
        
        let (mut write, mut read) = ws_stream.split();
        let mut messages_received = Vec::new();
        let mut success = true;
        let mut error_msg = None;
        
        // Обрабатываем сообщения
        for msg in &step.messages {
            // Отправляем сообщение
            if let Some(send_text) = &msg.send {
                let rendered = self.render_template(send_text, context);
                debug!("Отправляем: {}", rendered);
                
                if let Err(e) = write.send(Message::Text(rendered)).await {
                    error_msg = Some(format!("Ошибка отправки: {}", e));
                    success = false;
                    break;
                }
            }
            
            // Ждём ответ
            if let Some(wait) = msg.wait {
                tokio::time::sleep(wait).await;
            }
            
            // Проверяем ожидаемый ответ
            if let Some(expect_text) = &msg.expect {
                let timeout_read = step.timeout.unwrap_or(Duration::from_secs(10));
                let read_future = read.next();
                
                match tokio::time::timeout(timeout_read, read_future).await {
                    Ok(Some(Ok(message))) => {
                        let received = message.to_string();
                        messages_received.push(received.clone());
                        debug!("Получено: {}", received);
                        
                        if !received.contains(expect_text) {
                            error_msg = Some(format!(
                                "Ожидалось: {}, получено: {}",
                                expect_text, received
                            ));
                            success = false;
                            break;
                        }
                    }
                    Ok(Some(Err(e))) => {
                        error_msg = Some(format!("Ошибка чтения: {}", e));
                        success = false;
                        break;
                    }
                    Ok(None) => {
                        error_msg = Some("Соединение закрыто".to_string());
                        success = false;
                        break;
                    }
                    Err(_) => {
                        error_msg = Some("Таймаут ожидания ответа".to_string());
                        success = false;
                        break;
                    }
                }
            }
            
            // Проверяем JSONPath
            if let Some(jsonpath) = &msg.expect_jsonpath {
                let timeout_read = step.timeout.unwrap_or(Duration::from_secs(10));
                let read_future = read.next();
                
                match tokio::time::timeout(timeout_read, read_future).await {
                    Ok(Some(Ok(message))) => {
                        let received = message.to_string();
                        messages_received.push(received.clone());
                        debug!("Получено: {}", received);
                        
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&received) {
                            let mut selector = jsonpath_lib::selector(&json);
                            if let Ok(result) = selector(jsonpath) {
                                if result.is_empty() {
                                    error_msg = Some(format!("JSONPath '{}' не найден", jsonpath));
                                    success = false;
                                    break;
                                }
                            } else {
                                error_msg = Some(format!("Ошибка JSONPath: {}", jsonpath));
                                success = false;
                                break;
                            }
                        } else {
                            error_msg = Some(format!("Невалидный JSON: {}", received));
                            success = false;
                            break;
                        }
                    }
                    Ok(Some(Err(e))) => {
                        error_msg = Some(format!("Ошибка чтения: {}", e));
                        success = false;
                        break;
                    }
                    Ok(None) => {
                        error_msg = Some("Соединение закрыто".to_string());
                        success = false;
                        break;
                    }
                    Err(_) => {
                        error_msg = Some("Таймаут ожидания ответа".to_string());
                        success = false;
                        break;
                    }
                }
            }
        }
        
        let elapsed = start.elapsed();
        
        // Закрываем соединение
        let _ = write.close().await;
        
        Ok(WebSocketResponse {
            messages: messages_received,
            latency: elapsed,
            success,
            error: error_msg,
        })
    }

    fn render_template(&self, template: &str, context: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketResponse {
    pub messages: Vec<String>,
    pub latency: Duration,
    pub success: bool,
    pub error: Option<String>,
}