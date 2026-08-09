// src/executor/context.rs
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    inner: Arc<DashMap<String, String>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub fn set(&self, key: &str, value: String) {
        self.inner.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).map(|v| v.clone())
    }

    pub fn get_all(&self) -> HashMap<String, String> {
        self.inner.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect()
    }

    pub fn clear(&self) {
        self.inner.clear();
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}