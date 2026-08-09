// src/metrics/snapshot.rs
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Percentiles {
    pub p50: Duration,
    pub p75: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub p99_9: Duration,
}

impl Default for Percentiles {
    fn default() -> Self {
        Self {
            p50: Duration::ZERO,
            p75: Duration::ZERO,
            p90: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            p99_9: Duration::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_duration: Duration,
    pub average_latency: Duration,
    pub percentiles: Percentiles,
    pub rps: f64,
    pub error_rate: f64,
    pub total_errors: u64,
    pub status_codes: Vec<(u16, u64)>,
    pub success_rate: f64,
    pub requests_by_step: Vec<(String, u64)>,
}