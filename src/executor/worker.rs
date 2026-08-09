// src/executor/worker.rs
use crate::config::Step;
use crate::errors::BombardierResult;
use crate::protocols::http::HttpExecutor;
use super::context::Context;
use tracing::debug;

pub struct Worker {
    pub id: usize,  // Делаем публичным
    http_executor: HttpExecutor,
}

impl Worker {
    pub fn new(id: usize) -> BombardierResult<Self> {
        Ok(Self {
            id,
            http_executor: HttpExecutor::new()?,
        })
    }

    pub async fn execute_step(&self, step: &Step, context: &Context) -> BombardierResult<()> {
        debug!("Воркер {} выполняет шаг: {}", self.id, step.name);
        
        // Получаем все переменные из контекста
        let context_vars = context.get_all();
        
        // Выполняем HTTP запрос
        let response = self.http_executor.execute(step, &context_vars).await?;
        
        debug!("Воркер {} получил ответ: статус {}, время {}ms", 
            self.id, response.status, response.latency.as_millis());
        
        // Извлекаем переменные из ответа
        for extract in &step.extract {
            let value = if let Some(jsonpath) = &extract.jsonpath {
                // Парсим JSON и извлекаем по JSONPath
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
                // Извлекаем по Regex
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
                debug!("Воркер {} извлёк {} = {}", self.id, extract.name, value);
                context.set(&extract.name, value);
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