// src/protocols/http/client.rs
use crate::config::{Body, Method, Step};
use crate::errors::{BombardierError, BombardierResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Clone)]
pub struct HttpExecutor {
    client: reqwest::Client,
}

impl HttpExecutor {
    pub fn new() -> BombardierResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| BombardierError::Http(format!("Ошибка создания клиента: {}", e)))?;
        
        Ok(Self { client })
    }

    pub async fn execute(
        &self,
        step: &Step,
        context: &HashMap<String, String>,
    ) -> BombardierResult<HttpResponse> {
        let start = Instant::now();
        
        // Строим запрос
        let request = self.build_request(step, context)?;
        
        // Используем debug с форматированием
        let method_str = match step.method.as_ref().unwrap() {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
        };
        debug!("Выполняем запрос: {} {}", method_str, step.url);
        
        // Выполняем запрос с таймаутом
        let timeout = step.timeout.unwrap_or(Duration::from_secs(30));
        let response_future = self.client.execute(request);
        
        let response = tokio::time::timeout(timeout, response_future)
            .await
            .map_err(|_| BombardierError::Timeout(
                format!("Таймаут {}ms при выполнении запроса к {}", timeout.as_millis(), step.url)
            ))?
            .map_err(|e| BombardierError::Http(format!("Ошибка HTTP: {}", e)))?;
        
        let elapsed = start.elapsed();
        
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response.bytes()
            .await
            .map_err(|e| BombardierError::Http(format!("Ошибка чтения тела: {}", e)))?;
        
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();
        
        debug!("Ответ: статус {}, время: {}ms", status, elapsed.as_millis());
        
        Ok(HttpResponse {
            status: status.as_u16(),
            headers,
            body: body_bytes.to_vec(),
            body_str,
            latency: elapsed,
            success: status.is_success(),
        })
    }

    fn build_request(
        &self,
        step: &Step,
        context: &HashMap<String, String>,
    ) -> BombardierResult<reqwest::Request> {
        let method = step.method
            .as_ref()
            .ok_or_else(|| BombardierError::Http("Метод не указан".to_string()))?;
        
        // Подставляем переменные в URL
        let url = self.render_template(&step.url, context);
        
        // Создаём билдер запроса
        let mut builder = self.client.request(
            match method {
                Method::Get => reqwest::Method::GET,
                Method::Post => reqwest::Method::POST,
                Method::Put => reqwest::Method::PUT,
                Method::Delete => reqwest::Method::DELETE,
                Method::Patch => reqwest::Method::PATCH,
            },
            &url,
        );
        
        // Добавляем заголовки
        for (key, value) in &step.headers {
            let rendered_value = self.render_template(value, context);
            builder = builder.header(key, rendered_value);
        }
        
        // Добавляем тело
        match &step.body {
            Some(Body::Json(json)) => {
                let json_str = serde_json::to_string(json)
                    .map_err(|e| BombardierError::Http(format!("Ошибка сериализации JSON: {}", e)))?;
                let rendered = self.render_template(&json_str, context);
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(rendered);
            }
            Some(Body::Text(text)) => {
                let rendered = self.render_template(text, context);
                builder = builder.body(rendered);
            }
            Some(Body::Form(form)) => {
                let mut params = HashMap::new();
                for (key, value) in form {
                    let rendered_v = self.render_template(value, context);
                    params.insert(key.clone(), rendered_v);
                }
                builder = builder
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .form(&params);
            }
            None => {}
        };
        
        let request = builder.build()
            .map_err(|e| BombardierError::Http(format!("Ошибка сборки запроса: {}", e)))?;
        
        Ok(request)
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
pub struct HttpResponse {
    pub status: u16,
    pub headers: reqwest::header::HeaderMap,
    pub body: Vec<u8>,
    pub body_str: String,
    pub latency: Duration,
    pub success: bool,
}