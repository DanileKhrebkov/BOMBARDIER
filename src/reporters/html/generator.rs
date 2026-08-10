// src/reporters/html/generator.rs
use crate::config::Config;
use crate::metrics::MetricsSnapshot;
use std::fs;
use std::path::Path;

pub struct HtmlReporter;

impl HtmlReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export(&self, config: &Config, snapshot: &MetricsSnapshot, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let html = self.generate_html(config, snapshot);
        fs::write(path, html)?;
        Ok(())
    }

    fn generate_html(&self, config: &Config, snapshot: &MetricsSnapshot) -> String {
        let percentiles = &snapshot.percentiles;
        
        format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Отчёт Bombardier</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0a0e17;
            color: #e0e7f0;
            padding: 20px;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}
        .header {{
            background: linear-gradient(135deg, #1a2332 0%, #0f1520 100%);
            padding: 30px;
            border-radius: 12px;
            border: 1px solid #2a3a4a;
            margin-bottom: 30px;
        }}
        .header h1 {{
            font-size: 28px;
            font-weight: 700;
            background: linear-gradient(90deg, #60a5fa, #34d399);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        .header .subtitle {{
            color: #8899aa;
            margin-top: 8px;
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .card {{
            background: #141c2b;
            padding: 20px;
            border-radius: 10px;
            border: 1px solid #1e2d3d;
            transition: all 0.3s ease;
        }}
        .card:hover {{
            border-color: #2a4a6a;
            transform: translateY(-2px);
        }}
        .card .label {{
            font-size: 12px;
            text-transform: uppercase;
            color: #6a8a9a;
            letter-spacing: 0.5px;
            margin-bottom: 8px;
        }}
        .card .value {{
            font-size: 28px;
            font-weight: 700;
        }}
        .card .value.green {{ color: #34d399; }}
        .card .value.red {{ color: #f87171; }}
        .card .value.yellow {{ color: #fbbf24; }}
        .card .value.blue {{ color: #60a5fa; }}
        .card .value.purple {{ color: #a78bfa; }}
        .section {{
            background: #141c2b;
            padding: 25px;
            border-radius: 10px;
            border: 1px solid #1e2d3d;
            margin-bottom: 20px;
        }}
        .section h2 {{
            font-size: 20px;
            margin-bottom: 20px;
            color: #d0dbe8;
        }}
        .percentile-bar {{
            display: flex;
            align-items: center;
            gap: 15px;
            margin: 8px 0;
        }}
        .percentile-bar .label {{
            width: 40px;
            font-weight: 600;
            color: #6a8a9a;
        }}
        .percentile-bar .bar {{
            flex: 1;
            height: 24px;
            background: #1a2332;
            border-radius: 4px;
            overflow: hidden;
            position: relative;
        }}
        .percentile-bar .bar .fill {{
            height: 100%;
            background: linear-gradient(90deg, #34d399, #60a5fa);
            border-radius: 4px;
            transition: width 1s ease;
        }}
        .percentile-bar .value {{
            width: 80px;
            text-align: right;
            font-weight: 600;
            color: #d0dbe8;
        }}
        .status-codes {{
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }}
        .status-item {{
            background: #1a2332;
            padding: 12px 20px;
            border-radius: 8px;
            display: flex;
            align-items: center;
            gap: 12px;
        }}
        .status-item .code {{
            font-weight: 700;
            font-size: 18px;
        }}
        .status-item .code.success {{ color: #34d399; }}
        .status-item .code.error {{ color: #f87171; }}
        .status-item .count {{ color: #6a8a9a; }}
        .footer {{
            text-align: center;
            padding: 20px;
            color: #4a5a6a;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Bombardier</h1>
            <div class="subtitle">Отчёт о нагрузочном тестировании</div>
            <div style="margin-top: 12px; color: #6a8a9a;">
                <span>Тест: <strong style="color: #d0dbe8;">{}</strong></span>
                <span style="margin-left: 20px;">Длительность: <strong style="color: #d0dbe8;">{}с</strong></span>
                <span style="margin-left: 20px;">Воркеров: <strong style="color: #d0dbe8;">{}</strong></span>
                <span style="margin-left: 20px;">Запросов: <strong style="color: #d0dbe8;">{}</strong></span>
            </div>
        </div>

        <div class="grid">
            <div class="card">
                <div class="label">📊 RPS</div>
                <div class="value blue">{:.2}</div>
            </div>
            <div class="card">
                <div class="label">✅ Success Rate</div>
                <div class="value green">{:.2}%</div>
            </div>
            <div class="card">
                <div class="label">❌ Error Rate</div>
                <div class="value {}">{:.2}%</div>
            </div>
            <div class="card">
                <div class="label">⏱️ p95</div>
                <div class="value purple">{}ms</div>
            </div>
        </div>

        <div class="section">
            <h2>📈 Процентили времени ответа</h2>
            <div class="percentile-bar">
                <span class="label">p50</span>
                <div class="bar"><div class="fill" style="width: 50%"></div></div>
                <span class="value">{}ms</span>
            </div>
            <div class="percentile-bar">
                <span class="label">p75</span>
                <div class="bar"><div class="fill" style="width: 75%"></div></div>
                <span class="value">{}ms</span>
            </div>
            <div class="percentile-bar">
                <span class="label">p90</span>
                <div class="bar"><div class="fill" style="width: 90%"></div></div>
                <span class="value">{}ms</span>
            </div>
            <div class="percentile-bar">
                <span class="label">p95</span>
                <div class="bar"><div class="fill" style="width: 95%"></div></div>
                <span class="value">{}ms</span>
            </div>
            <div class="percentile-bar">
                <span class="label">p99</span>
                <div class="bar"><div class="fill" style="width: 99%"></div></div>
                <span class="value">{}ms</span>
            </div>
            <div class="percentile-bar">
                <span class="label">p99.9</span>
                <div class="bar"><div class="fill" style="width: 99.9%"></div></div>
                <span class="value">{}ms</span>
            </div>
        </div>

        <div class="section">
            <h2>📊 Статус коды</h2>
            <div class="status-codes">
                {}
            </div>
        </div>

        <div class="footer">
            Сгенерировано Bombardier v{} • {}
        </div>
    </div>
</body>
</html>"#,
            config.name,
            snapshot.total_duration.as_secs(),
            config.settings.workers,
            snapshot.total_requests,
            snapshot.rps,
            snapshot.success_rate,
            if snapshot.error_rate > 5.0 { "red" } else { "green" },
            snapshot.error_rate,
            snapshot.percentiles.p95.as_millis(),
            snapshot.percentiles.p50.as_millis(),
            snapshot.percentiles.p75.as_millis(),
            snapshot.percentiles.p90.as_millis(),
            snapshot.percentiles.p95.as_millis(),
            snapshot.percentiles.p99.as_millis(),
            snapshot.percentiles.p99_9.as_millis(),
            snapshot.status_codes.iter().map(|(code, count)| {
                format!(
                    r#"<div class="status-item">
                        <span class="code {}">{}</span>
                        <span class="count">{} ({}%)</span>
                    </div>"#,
                    if *code == 200 { "success" } else { "error" },
                    code,
                    count,
                    (*count as f64 / snapshot.total_requests as f64) * 100.0
                )
            }).collect::<String>(),
            env!("CARGO_PKG_VERSION"),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    }
}