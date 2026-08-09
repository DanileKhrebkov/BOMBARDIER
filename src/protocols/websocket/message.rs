// src/protocols/websocket/message.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketStep {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    pub messages: Vec<WebSocketMessage>,
    #[serde(default)]
    pub extract: Vec<WebSocketExtract>,
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    pub think_time: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(default)]
    pub send: Option<String>,
    #[serde(default)]
    pub expect: Option<String>,
    #[serde(default)]
    pub expect_jsonpath: Option<String>,
    #[serde(default)]
    pub expect_regex: Option<String>,
    #[serde(default, with = "humantime_serde")]
    pub wait: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketExtract {
    pub name: String,
    #[serde(default)]
    pub jsonpath: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
}

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