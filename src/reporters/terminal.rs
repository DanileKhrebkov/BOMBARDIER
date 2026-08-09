// src/reporters/terminal.rs
use crate::metrics::MetricsSnapshot;
use colored::*;

pub struct TerminalReporter;

impl TerminalReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, snapshot: &MetricsSnapshot) {
        println!("\n{}", "📊 РЕЗУЛЬТАТЫ ТЕСТА".bold().cyan());
        println!("{}", "=".repeat(50).cyan());

        // Общая статистика
        println!("\n{}", "📈 Общая статистика:".bold().yellow());
        println!("  Всего запросов: {}", snapshot.total_requests.to_string().bold().green());
        println!("  Успешных: {}", (snapshot.total_requests - snapshot.total_errors).to_string().bold().green());
        println!("  Ошибок: {}", snapshot.total_errors.to_string().bold().red());
        
        let success_color = if snapshot.success_rate > 95.0 { "green" } else if snapshot.success_rate > 80.0 { "yellow" } else { "red" };
        let error_color = if snapshot.error_rate < 1.0 { "green" } else if snapshot.error_rate < 5.0 { "yellow" } else { "red" };
        
        println!("  Success rate: {:.2}%", snapshot.success_rate.to_string().color(success_color).bold());
        println!("  Error rate: {:.2}%", snapshot.error_rate.to_string().color(error_color).bold());
        println!("  RPS: {:.2}", snapshot.rps.to_string().bold().cyan());

        // Время ответа
        println!("\n{}", "⏱️  Время ответа:".bold().yellow());
        println!("  Среднее: {}ms", snapshot.average_latency.as_millis().to_string().bold());
        println!("  p50: {}ms", snapshot.percentiles.p50.as_millis().to_string().bold());
        println!("  p75: {}ms", snapshot.percentiles.p75.as_millis().to_string().bold());
        println!("  p90: {}ms", snapshot.percentiles.p90.as_millis().to_string().bold());
        println!("  p95: {}ms", snapshot.percentiles.p95.as_millis().to_string().bold());
        println!("  p99: {}ms", snapshot.percentiles.p99.as_millis().to_string().bold());
        println!("  p99.9: {}ms", snapshot.percentiles.p99_9.as_millis().to_string().bold());

        // Статус коды
        if !snapshot.status_codes.is_empty() {
            println!("\n{}", "📊 Статус коды:".bold().yellow());
            for (code, count) in &snapshot.status_codes {
                let color = if *code == 200 { "green" } else if *code < 400 { "blue" } else if *code < 500 { "yellow" } else { "red" };
                let percentage = (*count as f64 / snapshot.total_requests as f64) * 100.0;
                println!("  {}: {} ({:.1}%)", code.to_string().color(color).bold(), count, percentage);
            }
        }

        // По шагам
        if !snapshot.requests_by_step.is_empty() {
            println!("\n{}", "📋 По шагам:".bold().yellow());
            for (step, count) in &snapshot.requests_by_step {
                let percentage = (*count as f64 / snapshot.total_requests as f64) * 100.0;
                println!("  {}: {} запросов ({:.1}%)", step.bold(), count, percentage);
            }
        }

        println!("\n{}", "=".repeat(50).cyan());
        println!("{}", "✅ Тест завершён!".bold().green());
    }
}