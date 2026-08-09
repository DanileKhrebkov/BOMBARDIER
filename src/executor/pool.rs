// src/executor/pool.rs
use crate::config::Config;
use crate::errors::BombardierResult;
use crate::executor::worker::Worker;
use crate::executor::context::Context;
use crate::metrics::{MetricsAggregator, MetricsCollector, Metric};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, debug, error};

pub struct Pool {
    config: Config,
    workers: Vec<Worker>,
    context: Context,
    metrics: Arc<MetricsAggregator>,
}

impl Pool {
    pub fn new(config: Config) -> BombardierResult<Self> {
        let mut workers = Vec::new();
        for i in 0..config.settings.workers {
            workers.push(Worker::new(i)?);
        }

        Ok(Self {
            config,
            workers,
            context: Context::new(),
            metrics: Arc::new(MetricsAggregator::new()),
        })
    }

    pub async fn run(&self) -> BombardierResult<()> {
        let worker_count = self.workers.len();
        info!("🚀 Запуск пула из {} воркеров", worker_count);

        let duration = self.config.settings.duration.unwrap_or(Duration::from_secs(30));
        info!("⏱️  Длительность теста: {}s", duration.as_secs());

        // Создаём канал для метрик
        let (collector, mut metrics_rx) = MetricsCollector::new();
        let metrics_clone = self.metrics.clone();

        // Запускаем поток для обработки метрик
        let metrics_handle = tokio::spawn(async move {
            while let Some(metric) = metrics_rx.recv().await {
                metrics_clone.record(metric).await;
            }
        });

        // Копируем шаги для всех воркеров
        let steps = self.config.steps.clone();
        let context = self.context.clone();

        // Запускаем воркеров
        let handles: Vec<_> = self.workers
            .iter()
            .map(|worker| {
                let steps = steps.clone();
                let context = context.clone();
                let sender = collector.get_sender();
                let duration = duration.clone();
                let worker_id = worker.id;

                tokio::spawn(async move {
                    let mut iteration = 0;
                    let start = std::time::Instant::now();

                    while start.elapsed() < duration {
                        iteration += 1;
                        debug!("Воркер {} итерация {}", worker_id, iteration);

                        // Выполняем все шаги
                        for step in &steps {
                            let step_name = step.name.clone();
                            let start_time = std::time::Instant::now();

                            // Временно создаём воркера для выполнения
                            let worker = Worker::new(worker_id).unwrap();
                            match worker.execute_step(step, &context).await {
                                Ok(_) => {
                                    let latency = start_time.elapsed();
                                    let metric = Metric {
                                        step_name: step_name.clone(),
                                        latency,
                                        status: 200,
                                        success: true,
                                        timestamp: std::time::Instant::now(),
                                    };
                                    let _ = sender.send(metric);
                                }
                                Err(e) => {
                                    error!("Воркер {} ошибка в шаге {}: {}", worker_id, step_name, e);
                                    let metric = Metric {
                                        step_name,
                                        latency: start_time.elapsed(),
                                        status: 500,
                                        success: false,
                                        timestamp: std::time::Instant::now(),
                                    };
                                    let _ = sender.send(metric);
                                }
                            }
                        }
                    }

                    debug!("Воркер {} завершил работу, итераций: {}", worker_id, iteration);
                })
            })
            .collect();

        // Ждём завершения всех воркеров
        for handle in handles {
            let _ = handle.await;
        }

        // Ждём завершения обработчика метрик
        // Просто дропаем sender, чтобы закрыть канал
        drop(collector);
        
        // Ждём пока обработчик метрик завершится
        let _ = metrics_handle.await;

        // Показываем результаты
        self.print_results().await;

        Ok(())
    }

    async fn print_results(&self) {
        let snapshot = self.metrics.snapshot().await;

        println!("\n📊 РЕЗУЛЬТАТЫ ТЕСТА");
        println!("{}", "=".repeat(50));

        println!("\n📈 Общая статистика:");
        println!("  Всего запросов: {}", snapshot.total_requests);
        println!("  Успешных: {}", snapshot.total_requests - snapshot.total_errors);
        println!("  Ошибок: {}", snapshot.total_errors);
        println!("  Success rate: {:.2}%", snapshot.success_rate);
        println!("  Error rate: {:.2}%", snapshot.error_rate);
        println!("  RPS: {:.2}", snapshot.rps);

        println!("\n⏱️  Время ответа:");
        println!("  Среднее: {}ms", snapshot.average_latency.as_millis());
        println!("  p50: {}ms", snapshot.percentiles.p50.as_millis());
        println!("  p75: {}ms", snapshot.percentiles.p75.as_millis());
        println!("  p90: {}ms", snapshot.percentiles.p90.as_millis());
        println!("  p95: {}ms", snapshot.percentiles.p95.as_millis());
        println!("  p99: {}ms", snapshot.percentiles.p99.as_millis());
        println!("  p99.9: {}ms", snapshot.percentiles.p99_9.as_millis());

        if !snapshot.status_codes.is_empty() {
            println!("\n📊 Статус коды:");
            for (code, count) in &snapshot.status_codes {
                let percentage = (*count as f64 / snapshot.total_requests as f64) * 100.0;
                println!("  {}: {} ({:.1}%)", code, count, percentage);
            }
        }

        if !snapshot.requests_by_step.is_empty() {
            println!("\n📋 По шагам:");
            for (step, count) in &snapshot.requests_by_step {
                println!("  {}: {} запросов", step, count);
            }
        }

        println!("\n{}", "=".repeat(50));
        println!("✅ Тест завершён!");
    }
}