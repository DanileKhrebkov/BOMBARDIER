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
    
    // Настройка логирования
    logging::init(cli.verbose);
    
    info!("🚀 Запуск Bombardier v{}", env!("CARGO_PKG_VERSION"));
    
    match cli.command {
        cli::Commands::Run(args) => {
            info!("Запуск теста с конфигом: {}", args.config.display());
            
            // Загружаем конфиг
            let config = config::load_config(&args.config)?;
            info!("✅ Конфиг загружен: {}", config.name);
            
            // Валидируем
            config::Validator::validate(&config)?;
            info!("✅ Конфиг валиден");
            
            if args.dry_run {
                println!("🔍 Dry-run режим: проверка выполнена успешно");
                println!("📋 Конфиг:");
                println!("  Название: {}", config.name);
                println!("  Воркеров: {}", config.settings.workers);
                if let Some(d) = config.settings.duration {
                    println!("  Длительность: {}s", d.as_secs());
                }
                if let Some(r) = config.settings.ramp_up {
                    println!("  Ramp-up: {}s", r.as_secs());
                }
                println!("  Шагов: {}", config.steps.len());
                for step in &config.steps {
                    println!("    - {} ({:?})", step.name, step.protocol);
                }
                if !config.assertions.is_empty() {
                    println!("  Ассертов: {}", config.assertions.len());
                }
                return Ok(());
            }
            
            // Создаём пул воркеров
            let pool = executor::Pool::new(config)?;
            
            // Запускаем тест
            pool.run().await?;
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
  duration: 10s
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
    think_time: 100ms
    timeout: 5s

assertions:
  - error_rate < 1%
  - p95 < 200ms
  - throughput > 5000 req/s
"#;
    
    fs::write(path, example)?;
    Ok(())
}