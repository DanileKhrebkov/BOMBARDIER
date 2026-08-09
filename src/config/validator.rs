// src/config/validator.rs
use super::models::{Config, Step, Protocol, Extract, Assertion};
use crate::errors::{BombardierError, BombardierResult};
use regex::Regex;
use std::collections::HashSet;

pub struct Validator;

impl Validator {
    pub fn validate(config: &Config) -> BombardierResult<()> {
        // Проверяем имя
        if config.name.is_empty() {
            return Err(BombardierError::Validation(
                "Имя конфига не может быть пустым".to_string()
            ));
        }

        // Проверяем шаги
        if config.steps.is_empty() {
            return Err(BombardierError::Validation(
                "Должен быть хотя бы один шаг".to_string()
            ));
        }

        // Проверяем каждый шаг
        let mut step_names = HashSet::new();
        for (idx, step) in config.steps.iter().enumerate() {
            Self::validate_step(step, idx)?;
            
            // Проверяем уникальность имён шагов
            if !step_names.insert(step.name.clone()) {
                return Err(BombardierError::Validation(
                    format!("Имя шага '{}' должно быть уникальным", step.name)
                ));
            }
        }

        // Проверяем ассерты
        for assertion in &config.assertions {
            Self::validate_assertion(assertion)?;
        }

        Ok(())
    }

    fn validate_step(step: &Step, idx: usize) -> BombardierResult<()> {
        // Проверяем URL
        if step.url.is_empty() {
            return Err(BombardierError::Validation(
                format!("Шаг {}: URL не может быть пустым", idx + 1)
            ));
        }

        // Проверяем метод для HTTP
        if matches!(step.protocol, Protocol::Http) {
            if step.method.is_none() {
                return Err(BombardierError::Validation(
                    format!("Шаг '{}': Для HTTP протокола должен быть указан метод", step.name)
                ));
            }
        }

        // Проверяем экстракты
        for extract in &step.extract {
            Self::validate_extract(extract, &step.name)?;
        }

        Ok(())
    }

    fn validate_extract(extract: &Extract, step_name: &str) -> BombardierResult<()> {
        if extract.jsonpath.is_none() && extract.regex.is_none() {
            return Err(BombardierError::Validation(
                format!(
                    "Шаг '{}': В экстракте '{}' должен быть указан jsonpath или regex",
                    step_name, extract.name
                )
            ));
        }

        // Проверяем синтаксис JSONPath
        if let Some(jsonpath) = &extract.jsonpath {
            if jsonpath.is_empty() {
                return Err(BombardierError::Validation(
                    format!("Шаг '{}': JSONPath не может быть пустым", step_name)
                ));
            }
            if !jsonpath.starts_with('$') {
                return Err(BombardierError::Validation(
                    format!("Шаг '{}': JSONPath должен начинаться с '$', получено: '{}'", step_name, jsonpath)
                ));
            }
        }

        // Проверяем синтаксис Regex
        if let Some(regex) = &extract.regex {
            if Regex::new(regex).is_err() {
                return Err(BombardierError::Validation(
                    format!("Шаг '{}': Невалидный Regex '{}'", step_name, regex)
                ));
            }
        }

        Ok(())
    }

    fn validate_assertion(assertion: &Assertion) -> BombardierResult<()> {
        match assertion {
            Assertion::Simple(s) => {
                // Парсим строку ассерта
                // Поддерживаем форматы:
                // - error_rate < 1%
                // - p95 < 200ms
                // - throughput > 5000 req/s (с пробелом в req/s)
                // - throughput > 5000req/s (без пробела)
                
                let s = s.trim();
                
                // Пробуем распарсить через регулярное выражение
                let re = Regex::new(r"^(\w+)\s*([<>]=?|==|!=)\s*(.+)$")
                    .map_err(|e| BombardierError::Validation(
                        format!("Ошибка парсинга ассерта: {}", e)
                    ))?;
                
                let captures = re.captures(s)
                    .ok_or_else(|| BombardierError::Validation(
                        format!("Невалидный формат ассерта '{}'. Ожидается: metric operator threshold", s)
                    ))?;
                
                let metric = captures.get(1).unwrap().as_str();
                let operator = captures.get(2).unwrap().as_str();
                let threshold = captures.get(3).unwrap().as_str().trim();
                
                // Проверяем метрику
                match metric {
                    "error_rate" | "p95" | "p99" | "throughput" | "total_requests" | "success_rate" => {}
                    _ => {
                        return Err(BombardierError::Validation(
                            format!("Неизвестная метрика '{}'. Доступные: error_rate, p95, p99, throughput, total_requests, success_rate", metric)
                        ));
                    }
                }
                
                // Проверяем оператор
                match operator {
                    "<" | ">" | "<=" | ">=" | "==" | "!=" => {}
                    _ => {
                        return Err(BombardierError::Validation(
                            format!("Невалидный оператор '{}'. Доступные: <, >, <=, >=, ==, !=", operator)
                        ));
                    }
                }
                
                // Проверяем порог
                if metric == "error_rate" || metric == "success_rate" {
                    if !threshold.ends_with('%') {
                        return Err(BombardierError::Validation(
                            format!("Для метрики '{}' порог должен быть в процентах (например: 1%)", metric)
                        ));
                    }
                    let num = threshold.trim_end_matches('%');
                    if num.parse::<f64>().is_err() {
                        return Err(BombardierError::Validation(
                            format!("Невалидное число в пороге '{}'", threshold)
                        ));
                    }
                } else if metric == "p95" || metric == "p99" {
                    if !threshold.ends_with("ms") && !threshold.ends_with('s') {
                        return Err(BombardierError::Validation(
                            format!("Для метрики '{}' порог должен быть временем (например: 200ms, 1s)", metric)
                        ));
                    }
                } else if metric == "throughput" {
                    // Поддерживаем "5000 req/s" и "5000req/s"
                    let threshold_clean = threshold.replace(' ', "");
                    if !threshold_clean.ends_with("req/s") {
                        return Err(BombardierError::Validation(
                            format!("Для метрики '{}' порог должен быть в req/s (например: 5000 req/s)", metric)
                        ));
                    }
                    let num = threshold_clean.trim_end_matches("req/s");
                    if num.parse::<f64>().is_err() {
                        return Err(BombardierError::Validation(
                            format!("Невалидное число в пороге '{}'", threshold)
                        ));
                    }
                }
                
                Ok(())
            }
            Assertion::Structured { metric, operator, threshold } => {
                // Валидируем структурированный ассерт
                if metric.is_empty() || operator.is_empty() || threshold.is_empty() {
                    return Err(BombardierError::Validation(
                        "Структурированный ассерт не может иметь пустые поля".to_string()
                    ));
                }
                Ok(())
            }
        }
    }
}