// src/reporters/progress.rs
use crate::metrics::MetricsAggregator;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[derive(Clone)]
pub struct ProgressReporter {
    metrics: Arc<MetricsAggregator>,
    progress_bar: ProgressBar,
    start_time: std::time::Instant,
    total_duration: Duration,
    errors: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl ProgressReporter {
    pub fn new(metrics: Arc<MetricsAggregator>, duration: Duration) -> Self {
        let progress_bar = ProgressBar::new(duration.as_secs());
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len}s {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        // Убираем постоянное обновление, чтобы не моргало
        progress_bar.disable_steady_tick();

        Self {
            metrics,
            progress_bar,
            start_time: std::time::Instant::now(),
            total_duration: duration,
            errors: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_millis(500));
        
        while self.start_time.elapsed() < self.total_duration {
            interval.tick().await;
            
            let snapshot = self.metrics.snapshot().await;
            let total = snapshot.total_requests;
            
            // Считаем RPS
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let rps = if elapsed > 0.0 {
                total as f64 / elapsed
            } else {
                0.0
            };

            // Обновляем прогресс-бар
            let elapsed_secs = elapsed as u64;
            if elapsed_secs <= self.total_duration.as_secs() {
                self.progress_bar.set_position(elapsed_secs);
            }

            // Формируем сообщение
            let error_rate = if total > 0 {
                (snapshot.total_errors as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            // Используем фиксированную ширину сообщения, чтобы не прыгало
            let msg = format!(
                "📊 {:>4} req | {:>3} err ({:>5.1}%) | {:>5.1} RPS | p95: {:>4}ms",
                total,
                snapshot.total_errors,
                error_rate,
                rps,
                snapshot.percentiles.p95.as_millis()
            );

            self.progress_bar.set_message(msg);
        }

        // Дожидаемся реального окончания
        while self.start_time.elapsed() < self.total_duration {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        self.progress_bar.finish_with_message("✅ Тест завершён!");
    }

    pub async fn add_error(&self, error: String) {
        let mut errors = self.errors.lock().await;
        errors.push(error);
        if errors.len() > 1000 {
            errors.truncate(1000);
        }
    }

    pub async fn get_errors(&self) -> Vec<String> {
        let errors = self.errors.lock().await;
        errors.clone()
    }

    pub fn finish(&self) {
        self.progress_bar.finish_and_clear();
    }
}