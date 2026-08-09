// src/config/template.rs
use crate::errors::BombardierResult;

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, template: &str, _context: &std::collections::HashMap<String, String>) -> BombardierResult<String> {
        // Пока просто возвращаем как есть
        Ok(template.to_string())
    }
}