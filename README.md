```markdown
# Bombardier

CLI-инструмент для нагрузочного тестирования HTTP/gRPC/WebSocket сервисов с поддержкой многошаговых сценариев.

## 🚀 Быстрый старт

### Установка

```bash
# Клонируем репозиторий
git clone https://github.com/yourusername/bombardier.git
cd bombardier

# Собираем
cargo build --release

# Бинарник будет в target/release/bombardier
```

### Простой пример

```bash
# Генерируем пример конфига
./target/release/bombardier generate --output test.yaml

# Запускаем тест
./target/release/bombardier run --config test.yaml

# С экспортом отчётов
./target/release/bombardier run --config test.yaml --json report.json --html report.html
```

## 📝 Конфигурация

### Базовый HTTP тест

```yaml
name: "Тест API"
settings:
  workers: 10
  duration: 30s
  ramp_up: 5s

steps:
  - name: get_posts
    protocol: http
    method: GET
    url: https://jsonplaceholder.typicode.com/posts
    headers:
      User-Agent: "Bombardier/1.0"
    timeout: 5s

assertions:
  - error_rate < 1%
  - p95 < 500ms
  - throughput > 100 req/s
```

### Многошаговый сценарий

```yaml
name: "API тест"
settings:
  workers: 5
  duration: 60s

steps:
  - name: login
    protocol: http
    method: POST
    url: https://api.example.com/auth/login
    body:
      username: "admin"
      password: "password123"
    extract:
      - name: token
        jsonpath: "$.token"

  - name: get_profile
    protocol: http
    method: GET
    url: https://api.example.com/profile
    headers:
      Authorization: "Bearer {{token}}"
    think_time: 1s

assertions:
  - error_rate < 1%
  - p95 < 1000ms
```

### WebSocket тест

```yaml
name: "WebSocket чат"
settings:
  workers: 3
  duration: 30s

steps:
  - name: websocket_chat
    protocol: websocket
    url: "wss://echo.websocket.org"
    messages:
      - send: "Hello WebSocket!"
        expect: "Hello WebSocket!"
        wait: 1s
      - send: '{"type": "ping"}'
        expect_jsonpath: "$.type"
        wait: 1s

assertions:
  - error_rate < 10%
  - p95 < 3000ms
```

### gRPC тест

```yaml
name: "gRPC сервис"
settings:
  workers: 5
  duration: 30s

steps:
  - name: grpc_call
    protocol: grpc
    url: "http://localhost:50051"
    grpc_method: "SayHello"
    grpc_request:
      name: "World"
    timeout: 5s

assertions:
  - error_rate < 1%
  - p95 < 500ms
```

## 📊 CLI Команды

### run - Запуск нагрузочного теста

```bash
bombardier run --config test.yaml

# С экспортом отчётов
bombardier run --config test.yaml --json report.json --html report.html

# Переопределение параметров
bombardier run --config test.yaml --workers 20 --duration 60s

# Dry-run режим (проверка без выполнения)
bombardier run --config test.yaml --dry-run
```

### validate - Проверка конфига

```bash
bombardier validate --config test.yaml

# С детальным выводом
bombardier validate --config test.yaml --detail
```

### generate - Генерация примера конфига

```bash
bombardier generate --output my-config.yaml

# Тип конфига (http, grpc, websocket, multi)
bombardier generate --kind http --output test.yaml
```

## 📈 Отчёты

### Терминальный отчёт

```
📊 РЕЗУЛЬТАТЫ ТЕСТА
==================================================

📈 Общая статистика:
  Всего запросов: 1446
  Успешных: 1417
  Ошибок: 29
  Success rate: 97.99%
  Error rate: 2.01%
  RPS: 21.68

⏱️  Время ответа:
  Среднее: 2118ms
  p50: 1875ms
  p75: 2590ms
  p90: 3360ms
  p95: 4249ms
  p99: 6580ms
  p99.9: 7932ms
```

### JSON экспорт

```bash
bombardier run --config test.yaml --json report.json
```

### HTML экспорт

```bash
bombardier run --config test.yaml --html report.html
```

## ⚙️ Конфигурация

### Settings

| Поле | Тип | Описание | По умолчанию |
|------|-----|----------|--------------|
| `workers` | number | Количество параллельных воркеров | 10 |
| `duration` | string | Длительность теста (30s, 5m, 1h) | - |
| `ramp_up` | string | Время плавного увеличения нагрузки | - |

### Step

| Поле | Тип | Описание |
|------|-----|----------|
| `name` | string | Название шага |
| `protocol` | string | Протокол: http, grpc, websocket |
| `url` | string | URL запроса |
| `method` | string | HTTP метод: GET, POST, PUT, DELETE, PATCH |
| `headers` | object | HTTP заголовки |
| `body` | object/string | Тело запроса |
| `extract` | array | Извлечение данных из ответа |
| `timeout` | string | Таймаут запроса |
| `think_time` | string | Пауза после выполнения |

### Extract

```yaml
extract:
  - name: token
    jsonpath: "$.data.token"
  - name: user_id
    regex: "User ID: (\\d+)"
```

### Assertions

```yaml
assertions:
  - error_rate < 1%
  - p95 < 200ms
  - throughput > 1000 req/s
  - total_requests > 10000
  - success_rate > 99%
```

## 🛠️ Системные требования

- Rust 1.70+
- Tokio runtime
- Поддержка TLS (для HTTPS/WSS)

## 📦 Зависимости

- `tokio` - Асинхронный runtime
- `clap` - CLI парсер
- `reqwest` - HTTP клиент
- `tonic` - gRPC клиент
- `tokio-tungstenite` - WebSocket клиент
- `serde` - Сериализация конфигов
- `indicatif` - Прогресс-бар
- `plotters` - Генерация графиков

## 🤝 Лицензия

MIT
