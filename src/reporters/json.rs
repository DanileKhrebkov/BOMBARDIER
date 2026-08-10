// src/reporters/json.rs
use crate::config::Config;
use crate::metrics::MetricsSnapshot;
use serde_json::json;
use std::fs;
use std::path::Path;

pub struct JsonReporter;

impl JsonReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export(&self, config: &Config, snapshot: &MetricsSnapshot, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Собираем результаты ассертов
        let assertion_results: Vec<_> = config.assertions.iter().map(|a| {
            json!({
                "condition": a.as_string(),
                "passed": false, // TODO: парсить результаты ассертов
            })
        }).collect();

        let json = json!({
            "test": {
                "name": config.name,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "duration_seconds": snapshot.total_duration.as_secs(),
                "workers": config.settings.workers,
            },
            "results": {
                "total_requests": snapshot.total_requests,
                "successful_requests": snapshot.total_requests - snapshot.total_errors,
                "failed_requests": snapshot.total_errors,
                "success_rate": snapshot.success_rate,
                "error_rate": snapshot.error_rate,
                "rps": snapshot.rps,
                "average_latency_ms": snapshot.average_latency.as_millis(),
                "percentiles": {
                    "p50": snapshot.percentiles.p50.as_millis(),
                    "p75": snapshot.percentiles.p75.as_millis(),
                    "p90": snapshot.percentiles.p90.as_millis(),
                    "p95": snapshot.percentiles.p95.as_millis(),
                    "p99": snapshot.percentiles.p99.as_millis(),
                    "p99_9": snapshot.percentiles.p99_9.as_millis(),
                },
                "status_codes": snapshot.status_codes.iter().map(|(code, count)| {
                    json!({
                        "code": code,
                        "count": count,
                        "percentage": (*count as f64 / snapshot.total_requests as f64) * 100.0
                    })
                }).collect::<Vec<_>>(),
                "steps": snapshot.requests_by_step.iter().map(|(name, count)| {
                    json!({
                        "name": name,
                        "requests": count,
                        "percentage": (*count as f64 / snapshot.total_requests as f64) * 100.0
                    })
                }).collect::<Vec<_>>(),
            },
            "assertions": assertion_results,
        });

        let content = serde_json::to_string_pretty(&json)?;
        fs::write(path, content)?;
        
        Ok(())
    }
}