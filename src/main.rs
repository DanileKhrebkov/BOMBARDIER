// src/main.rs
mod cli;
mod config;
mod executor;
mod protocols;
mod generator;
mod metrics;
mod reporters;
mod assertions;
mod errors;
mod logging;
mod utils;

use clap::Parser;
use tracing::info;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    
    logging::init(cli.verbose);
    
    info!("🚀 Запуск Bombardier v{}", env!("CARGO_PKG_VERSION"));
    
    match cli.command {
        cli::Commands::Run(args) => {
            info!("Запуск теста с конфигом: {}", args.config.display());
            
            let config = config::load_config(&args.config)?;
            info!("✅ Конфиг загружен: {}", config.name);
            
            config::Validator::validate(&config)?;
            info!("✅ Конфиг валиден");
            
            if args.dry_run {
                println!("🔍 Dry-run режим: проверка выполнена успешно");
                return Ok(());
            }
            
            let pool = executor::Pool::new(config.clone())?;
            
            // Запускаем тест, но не выходим при ошибке ассертов
            let test_result = pool.run().await;
            
            // ВСЕГДА получаем snapshot
            let snapshot = pool.get_snapshot().await;
            
            // Экспортируем JSON
            if let Some(json_path) = args.json {
                let reporter = reporters::JsonReporter::new();
                if let Err(e) = reporter.export(&config, &snapshot, &json_path) {
                    eprintln!("❌ Ошибка экспорта JSON: {}", e);
                } else {
                    println!("✅ JSON отчёт сохранён: {}", json_path.display());
                }
            }
            
            // Экспортируем HTML
            if let Some(html_path) = args.html {
                let reporter = reporters::HtmlReporter::new();
                if let Err(e) = reporter.export(&config, &snapshot, &html_path) {
                    eprintln!("❌ Ошибка экспорта HTML: {}", e);
                } else {
                    println!("✅ HTML отчёт сохранён: {}", html_path.display());
                }
            }
            
            // Теперь возвращаем ошибку если была
            test_result?;
        }
        cli::Commands::Validate(args) => {
            info!("Проверка конфига: {}", args.config.display());
            
            let config = config::load_config(&args.config)?;
            config::Validator::validate(&config)?;
            
            println!("✅ Конфиг валиден!");
            if args.detail {
                println!("\n📋 Детальная информация:");
                println!("  Название: {}", config.name);
                println!("  Воркеров: {}", config.settings.workers);
                if let Some(d) = config.settings.duration {
                    println!("  Длительность: {}s", d.as_secs());
                }
                if let Some(r) = config.settings.ramp_up {
                    println!("  Ramp-up: {}s", r.as_secs());
                }
                println!("  Шагов: {}", config.steps.len());
                for (i, step) in config.steps.iter().enumerate() {
                    println!("\n  Шаг {}: {}", i + 1, step.name);
                    println!("    Протокол: {:?}", step.protocol);
                    println!("    URL: {}", step.url);
                    if let Some(m) = &step.method {
                        println!("    Метод: {:?}", m);
                    }
                    if !step.headers.is_empty() {
                        println!("    Заголовки: {:?}", step.headers);
                    }
                    if let Some(b) = &step.body {
                        println!("    Body: {:?}", b);
                    }
                    if !step.extract.is_empty() {
                        println!("    Экстракты:");
                        for ext in &step.extract {
                            if let Some(jp) = &ext.jsonpath {
                                println!("      - {} (JSONPath: {})", ext.name, jp);
                            }
                            if let Some(re) = &ext.regex {
                                println!("      - {} (Regex: {})", ext.name, re);
                            }
                        }
                    }
                    if !step.messages.is_empty() {
                        println!("    WebSocket сообщения:");
                        for msg in &step.messages {
                            if let Some(send) = &msg.send {
                                println!("      - send: {}", send);
                            }
                            if let Some(expect) = &msg.expect {
                                println!("        expect: {}", expect);
                            }
                            if let Some(jp) = &msg.expect_jsonpath {
                                println!("        expect_jsonpath: {}", jp);
                            }
                        }
                    }
                    if let Some(t) = step.timeout {
                        println!("    Таймаут: {}ms", t.as_millis());
                    }
                    if let Some(t) = step.think_time {
                        println!("    Think time: {}ms", t.as_millis());
                    }
                }
                if !config.assertions.is_empty() {
                    println!("\n  Ассерты:");
                    for assertion in &config.assertions {
                        match assertion {
                            config::Assertion::Simple(s) => println!("    - {}", s),
                            config::Assertion::Structured { metric, operator, threshold } => {
                                println!("    - {} {} {}", metric, operator, threshold);
                            }
                        }
                    }
                }
            }
        }
        cli::Commands::Generate(args) => {
            info!("Генерация примера конфига: {}", args.output.display());
            generate_example_config(&args.output)?;
            println!("✅ Пример конфига сохранён в: {}", args.output.display());
        }
    }
    
    Ok(())
}

fn generate_example_config(path: &Path) -> anyhow::Result<()> {
    use std::fs;
    
    let example = r#"name: "Тест API блога"
settings:
  workers: 10
  duration: 30s
  ramp_up: 5s

steps:
  - name: login
    protocol: http
    method: POST
    url: https://httpbin.org/post
    body:
      username: "admin"
      password: "password123"
    extract:
      - name: token
        jsonpath: "$.json"

  - name: get_posts
    protocol: http
    method: GET
    url: https://httpbin.org/get
    headers:
      Authorization: "Bearer {{token}}"
    think_time: 1s
    timeout: 5s

assertions:
  - error_rate < 1%
  - p95 < 200ms
  - throughput > 5000 req/s
"#;
    
    fs::write(path, example)?;
    Ok(())
}