// src/executor/pool.rs
use crate::config::Config;
use crate::errors::BombardierResult;
use crate::executor::worker::Worker;
use crate::executor::context::Context;
use crate::metrics::{MetricsAggregator, MetricsCollector, Metric};
use crate::reporters::{ProgressReporter, TerminalReporter};
use crate::assertions::AssertionEvaluator;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, debug};
use colored::Colorize;

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

        // Запускаем прогресс-бар
        let progress_metrics = self.metrics.clone();
        let progress_reporter = ProgressReporter::new(progress_metrics, duration);
        let progress_handle = tokio::spawn({
            let reporter = progress_reporter.clone();
            async move {
                reporter.run().await;
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
                let progress_reporter = progress_reporter.clone();

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
                                    // Не выводим ошибку в консоль, а сохраняем
                                    let error_msg = format!(
                                        "Воркер {} ошибка в шаге {}: {}",
                                        worker_id, step_name, e
                                    );
                                    progress_reporter.add_error(error_msg).await;
                                    
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
        drop(collector);
        let _ = metrics_handle.await;

        // Ждём завершения прогресс-бара
        let _ = progress_handle.await;

        // Показываем ошибки если есть
        let errors = progress_reporter.get_errors().await;
        if !errors.is_empty() {
            println!("\n{}", "⚠️  Ошибки во время выполнения:".bold().yellow());
            for error in errors.iter().take(20) {
                println!("  • {}", error);
            }
            if errors.len() > 20 {
                println!("  ... и ещё {} ошибок", errors.len() - 20);
            }
        }

        // Показываем финальные результаты
        let snapshot = self.metrics.snapshot().await;
        let reporter = TerminalReporter::new();
        reporter.print(&snapshot);

        // Проверяем ассерты
        if !self.config.assertions.is_empty() {
            println!("\n{}", "🔍 Проверка условий...".bold().cyan());
            let results = AssertionEvaluator::evaluate(&self.config.assertions, &snapshot)?;
            let all_passed = AssertionEvaluator::print_results(&results);
            
            if !all_passed {
                return Err(crate::errors::BombardierError::Assertion(
                    "Некоторые проверки не пройдены".to_string()
                ));
            }
        }

        Ok(())
    }
}