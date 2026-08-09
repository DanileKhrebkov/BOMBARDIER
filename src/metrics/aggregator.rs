// src/metrics/aggregator.rs
use crate::metrics::collector::Metric;
use crate::metrics::snapshot::{MetricsSnapshot, Percentiles};
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MetricsAggregator {
    latencies: Arc<Mutex<Vec<Duration>>>,
    status_codes: Arc<DashMap<u16, u64>>,
    success_count: Arc<DashMap<String, u64>>,
    error_count: Arc<DashMap<String, u64>>,
    total_requests: Arc<DashMap<String, u64>>,
    start_time: Instant,
    window_size: usize,
    rps_window: Arc<Mutex<VecDeque<(Instant, u64)>>>,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            latencies: Arc::new(Mutex::new(Vec::new())),
            status_codes: Arc::new(DashMap::new()),
            success_count: Arc::new(DashMap::new()),
            error_count: Arc::new(DashMap::new()),
            total_requests: Arc::new(DashMap::new()),
            start_time: Instant::now(),
            window_size: 1000,
            rps_window: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn record(&self, metric: Metric) {
        // Записываем латентность
        {
            let mut latencies = self.latencies.lock().await;
            latencies.push(metric.latency);
            // Ограничиваем размер для памяти
            if latencies.len() > 100_000 {
                let drain = latencies.len() - 100_000;
                latencies.drain(0..drain);
            }
        }

        // Статус коды
        *self.status_codes.entry(metric.status).or_insert(0) += 1;

        // Успешность
        let step_key = metric.step_name.clone();
        if metric.success {
            *self.success_count.entry(step_key.clone()).or_insert(0) += 1;
        } else {
            *self.error_count.entry(step_key.clone()).or_insert(0) += 1;
        }

        // Общее количество
        *self.total_requests.entry(step_key).or_insert(0) += 1;

        // RPS окно
        {
            let mut window = self.rps_window.lock().await;
            window.push_back((Instant::now(), 1));
            // Оставляем только последнюю секунду
            let now = Instant::now();
            while let Some(&(time, _)) = window.front() {
                if now.duration_since(time) > Duration::from_secs(1) {
                    window.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    pub async fn snapshot(&self) -> MetricsSnapshot {
        let latencies = self.latencies.lock().await;
        let total = latencies.len() as u64;
        let total_duration = self.start_time.elapsed();

        // Вычисляем процентили
        let mut sorted = latencies.clone();
        sorted.sort();

        let percentiles = if total > 0 {
            Percentiles {
                p50: Self::percentile(&sorted, 50.0),
                p75: Self::percentile(&sorted, 75.0),
                p90: Self::percentile(&sorted, 90.0),
                p95: Self::percentile(&sorted, 95.0),
                p99: Self::percentile(&sorted, 99.0),
                p99_9: Self::percentile(&sorted, 99.9),
            }
        } else {
            Percentiles::default()
        };

        // RPS
        let rps = if total_duration.as_secs() > 0 {
            total as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        // Ошибки
        let total_errors: u64 = self.error_count.iter().map(|e| *e.value()).sum();
        let error_rate = if total > 0 {
            (total_errors as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Копируем статусы
        let status_codes: Vec<(u16, u64)> = self.status_codes
            .iter()
            .map(|e| (*e.key(), *e.value()))
            .collect();

        // Общее количество по шагам
        let requests_by_step: Vec<(String, u64)> = self.total_requests
            .iter()
            .map(|e| (e.key().clone(), *e.value()))
            .collect();

        MetricsSnapshot {
            total_requests: total,
            total_duration,
            average_latency: if total > 0 {
                latencies.iter().sum::<Duration>() / total as u32
            } else {
                Duration::ZERO
            },
            percentiles,
            rps,
            error_rate,
            total_errors,
            status_codes,
            success_rate: if total > 0 {
                ((total - total_errors) as f64 / total as f64) * 100.0
            } else {
                100.0
            },
            requests_by_step,
        }
    }

    fn percentile(sorted: &[Duration], p: f64) -> Duration {
        if sorted.is_empty() {
            return Duration::ZERO;
        }
        let index = (p / 100.0) * (sorted.len() - 1) as f64;
        let index_floor = index.floor() as usize;
        let index_ceil = index.ceil() as usize;
        
        if index_floor == index_ceil {
            sorted[index_floor]
        } else {
            let low = sorted[index_floor].as_nanos() as f64;
            let high = sorted[index_ceil].as_nanos() as f64;
            let frac = index - index_floor as f64;
            Duration::from_nanos((low + (high - low) * frac) as u64)
        }
    }

    pub fn get_total_requests(&self) -> u64 {
        self.total_requests.iter().map(|e| *e.value()).sum()
    }

    pub fn get_errors(&self) -> u64 {
        self.error_count.iter().map(|e| *e.value()).sum()
    }

    pub fn get_success_count(&self) -> u64 {
        self.success_count.iter().map(|e| *e.value()).sum()
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}