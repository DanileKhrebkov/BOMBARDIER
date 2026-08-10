// src/config/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// Модуль для humantime сериализации прямо в файле
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;
    use humantime::parse_duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => serializer.serialize_str(&humantime::format_duration(*d).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => parse_duration(&s).map(Some).map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    #[serde(default)]
    pub settings: Settings,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default, with = "humantime_serde")]
    pub duration: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    pub ramp_up: Option<Duration>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            workers: default_workers(),
            duration: None,
            ramp_up: None,
        }
    }
}

fn default_workers() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub protocol: Protocol,
    #[serde(default)]
    pub method: Option<Method>,  // Только для HTTP
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<Body>,
    #[serde(default)]
    pub extract: Vec<Extract>,
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    pub think_time: Option<Duration>,
    // WebSocket
    #[serde(default)]
    pub messages: Vec<crate::protocols::websocket::WebSocketMessage>,
    // gRPC
    #[serde(default)]
    pub grpc_method: Option<String>,
    #[serde(default)]
    pub grpc_request: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Grpc,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Body {
    Json(serde_json::Value),
    Text(String),
    Form(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extract {
    pub name: String,
    #[serde(default)]
    pub jsonpath: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Assertion {
    Simple(String),
    Structured {
        metric: String,
        operator: String,
        threshold: String,
    },
}

impl Assertion {
    pub fn as_string(&self) -> String {
        match self {
            Assertion::Simple(s) => s.clone(),
            Assertion::Structured { metric, operator, threshold } => {
                format!("{} {} {}", metric, operator, threshold)
            }
        }
    }
}