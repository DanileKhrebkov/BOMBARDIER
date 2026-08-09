// src/config/loader.rs
use super::models::Config;
use crate::errors::BombardierResult;
use std::fs;
use std::path::Path;

pub fn load_config(path: &Path) -> BombardierResult<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn load_config_from_str(content: &str) -> BombardierResult<Config> {
    let config: Config = serde_yaml::from_str(content)?;
    Ok(config)
}