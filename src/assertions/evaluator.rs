// src/assertions/evaluator.rs
use crate::config::Assertion;
use crate::metrics::MetricsSnapshot;
use crate::errors::{BombardierError, BombardierResult};
use super::conditions::{AssertionResult, AssertionStatus};
use regex::Regex;
use colored::Colorize;

pub struct AssertionEvaluator;

impl AssertionEvaluator {
    pub fn evaluate(assertions: &[Assertion], snapshot: &MetricsSnapshot) -> BombardierResult<Vec<AssertionResult>> {
        let mut results = Vec::new();

        for assertion in assertions {
            let result = match assertion {
                Assertion::Simple(s) => Self::evaluate_simple(s, snapshot)?,
                Assertion::Structured { metric, operator, threshold } => {
                    Self::evaluate_structured(metric, operator, threshold, snapshot)?
                }
            };
            results.push(result);
        }

        Ok(results)
    }

    fn evaluate_simple(assertion_str: &str, snapshot: &MetricsSnapshot) -> BombardierResult<AssertionResult> {
        // Парсим строку ассерта
        let re = Regex::new(r"^(\w+)\s*([<>]=?|==|!=)\s*(.+)$")
            .map_err(|e| BombardierError::Assertion(format!("Ошибка парсинга ассерта: {}", e)))?;

        let captures = re.captures(assertion_str)
            .ok_or_else(|| BombardierError::Assertion(
                format!("Невалидный формат ассерта '{}'", assertion_str)
            ))?;

        let metric = captures.get(1).unwrap().as_str();
        let operator = captures.get(2).unwrap().as_str();
        let threshold = captures.get(3).unwrap().as_str().trim();

        Self::evaluate_structured(metric, operator, threshold, snapshot)
    }

    fn evaluate_structured(
        metric: &str,
        operator: &str,
        threshold: &str,
        snapshot: &MetricsSnapshot,
    ) -> BombardierResult<AssertionResult> {
        // Получаем фактическое значение
        let actual_value = match metric {
            "error_rate" => snapshot.error_rate,  // Уже в процентах
            "p95" => snapshot.percentiles.p95.as_millis() as f64,
            "p99" => snapshot.percentiles.p99.as_millis() as f64,
            "throughput" => snapshot.rps,
            "total_requests" => snapshot.total_requests as f64,
            "success_rate" => snapshot.success_rate,  // Уже в процентах
            _ => {
                return Err(BombardierError::Assertion(
                    format!("Неизвестная метрика '{}'", metric)
                ));
            }
        };

        // Парсим порог
        let threshold_value = Self::parse_threshold(threshold, metric)?;
        let threshold_str = threshold.to_string();

        // Проверяем условие
        let (pass, actual_str) = match operator {
            "<" => {
                let pass = actual_value < threshold_value;
                let actual_str = Self::format_value(actual_value, metric);
                (pass, actual_str)
            }
            ">" => {
                let pass = actual_value > threshold_value;
                let actual_str = Self::format_value(actual_value, metric);
                (pass, actual_str)
            }
            "<=" => {
                let pass = actual_value <= threshold_value;
                let actual_str = Self::format_value(actual_value, metric);
                (pass, actual_str)
            }
            ">=" => {
                let pass = actual_value >= threshold_value;
                let actual_str = Self::format_value(actual_value, metric);
                (pass, actual_str)
            }
            "==" => {
                let pass = (actual_value - threshold_value).abs() < f64::EPSILON;
                let actual_str = Self::format_value(actual_value, metric);
                (pass, actual_str)
            }
            "!=" => {
                let pass = (actual_value - threshold_value).abs() >= f64::EPSILON;
                let actual_str = Self::format_value(actual_value, metric);
                (pass, actual_str)
            }
            _ => {
                return Err(BombardierError::Assertion(
                    format!("Неизвестный оператор '{}'", operator)
                ));
            }
        };

        let assertion_str = format!("{} {} {}", metric, operator, threshold_str);
        
        if pass {
            Ok(AssertionResult::pass(
                assertion_str,
                actual_str.clone(),
                threshold_str,
            ))
        } else {
            Ok(AssertionResult::fail(
                assertion_str,
                actual_str.clone(),
                threshold_str,
                format!("Ожидалось: {} {}, получено: {}", metric, operator, actual_str)
            ))
        }
    }

    fn parse_threshold(threshold: &str, metric: &str) -> BombardierResult<f64> {
        let threshold_clean = threshold.replace(' ', "");

        let value = if metric == "error_rate" || metric == "success_rate" {
            // Убираем %
            let num = threshold_clean.trim_end_matches('%');
            num.parse::<f64>()
                .map_err(|_| BombardierError::Assertion(
                    format!("Невалидное число в пороге '{}'", threshold)
                ))?
        } else if metric == "p95" || metric == "p99" {
            // Убираем ms или s
            let num = if threshold_clean.ends_with("ms") {
                threshold_clean.trim_end_matches("ms")
            } else if threshold_clean.ends_with('s') {
                threshold_clean.trim_end_matches('s')
            } else {
                threshold
            };
            num.parse::<f64>()
                .map_err(|_| BombardierError::Assertion(
                    format!("Невалидное число в пороге '{}'", threshold)
                ))?
        } else if metric == "throughput" {
            // Убираем req/s
            let num = threshold_clean.trim_end_matches("req/s");
            num.parse::<f64>()
                .map_err(|_| BombardierError::Assertion(
                    format!("Невалидное число в пороге '{}'", threshold)
                ))?
        } else {
            threshold_clean.parse::<f64>()
                .map_err(|_| BombardierError::Assertion(
                    format!("Невалидное число в пороге '{}'", threshold)
                ))?
        };

        Ok(value)
    }

    fn format_value(value: f64, metric: &str) -> String {
        match metric {
            "error_rate" | "success_rate" => format!("{:.2}%", value),
            "p95" | "p99" => {
                if value >= 1000.0 {
                    format!("{:.2}s", value / 1000.0)
                } else {
                    format!("{:.0}ms", value)
                }
            }
            "throughput" => format!("{:.2} req/s", value),
            "total_requests" => format!("{:.0}", value),
            _ => format!("{:.2}", value),
        }
    }

    pub fn print_results(results: &[AssertionResult]) -> bool {
        println!("\n{}", "🔍 РЕЗУЛЬТАТЫ ПРОВЕРОК".bold().cyan());
        println!("{}", "=".repeat(50).cyan());

        let mut all_passed = true;

        for result in results {
            let status_str = match result.status {
                AssertionStatus::Pass => result.status.to_string().green().bold(),
                AssertionStatus::Fail => {
                    all_passed = false;
                    result.status.to_string().red().bold()
                }
            };

            println!("\n  {}: {}", status_str, result.assertion.bold());
            println!("    Ожидалось: {}", result.expected_value);
            println!("    Получено: {}", result.actual_value);
            if matches!(result.status, AssertionStatus::Fail) {
                println!("    {}", result.message.red());
            }
        }

        println!("\n{}", "=".repeat(50).cyan());

        if all_passed {
            println!("{}", "✅ ВСЕ ПРОВЕРКИ ПРОЙДЕНЫ!".bold().green());
        } else {
            println!("{}", "❌ ЕСТЬ ПРОВАЛЕННЫЕ ПРОВЕРКИ!".bold().red());
        }

        all_passed
    }
}