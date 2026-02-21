# Server

Веб-сервер блог-платформы с поддержкой HTTP и gRPC API.

## Архитектура

Проект следует принципам Clean Architecture:

- **domain/** - доменные сущности и бизнес-правила
- **application/** - use cases и бизнес-логика
- **data/** - репозитории и работа с БД
- **infrastructure/** - конфигурация и внешние сервисы
- **presentation/** - HTTP и gRPC handlers

## Запуск

### Требования

- PostgreSQL 14+
- Rust 1.75+

### Конфигурация

Создайте файл `config.yml` в директории `server/`:

```yaml
db_connection_string: postgresql://pguser:pgdata@localhost:5432/pgdb
jwt_secret: your-secret-key-here-min-64-chars
jwt_expiration_seconds: 3600
server_port: 8080
grpc_port: 50051
cors_origin: http://localhost:3000
log_level: info
```

**Параметры:**

- `db_connection_string` - строка подключения к PostgreSQL
- `jwt_secret` - секретный ключ для подписи JWT (минимум 64 символа)
- `jwt_expiration_seconds` - время жизни access токена в секундах
- `server_port` - порт HTTP сервера
- `grpc_port` - порт gRPC сервера
- `cors_origin` - разрешённый origin для CORS
- `log_level` - уровень логирования (trace, debug, info, warn, error)

Пример конфигурации: `config.yaml.example`

### Запуск сервера

```bash
# Из корня проекта
cargo run --bin server

# Или из директории server/
cd server
cargo run
```

Сервер запустится на:

- HTTP API: `http://localhost:8080`
- gRPC API: `http://localhost:50051`

## API Endpoints

### HTTP REST API

**Аутентификация (публичные):**

- `POST /api/v1/auth/register` - регистрация пользователя
- `POST /api/v1/auth/login` - вход в систему
- `POST /api/v1/auth/refresh` - обновление токена

**Посты:**

- `GET /api/v1/posts` - список постов (публичный)
- `GET /api/v1/posts/{id}` - получить пост (публичный)
- `POST /api/v1/posts` - создать пост (требует auth)
- `PUT /api/v1/posts/{id}` - обновить пост (требует auth)
- `DELETE /api/v1/posts/{id}` - удалить пост (требует auth)

### gRPC API

Все методы из protobuf схемы `api/proto/blog.proto`:

- `Register` - регистрация
- `Login` - вход
- `RefreshToken` - обновление токена
- `CreatePost` - создание поста
- `GetPost` - получение поста
- `UpdatePost` - обновление поста
- `DeletePost` - удаление поста
- `ListPosts` - список постов

## База данных

### Миграции

Миграции находятся в `migrations/`. Применяются вручную:

```bash
# Установить sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Применить миграции
sqlx migrate run --database-url postgresql://pguser:pgdata@localhost:5432/pgdb
```

### Схема

```sql
-- Пользователи
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Посты
CREATE TABLE posts (
    id UUID PRIMARY KEY,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Разработка

```bash
# Запустить тесты
cargo test

# Проверить код
cargo clippy

# Запустить с отладочными логами
RUST_LOG=debug cargo run
```

## Примеры запросов

### Регистрация

```bash
curl -X POST http://localhost:8081/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"secret123"}'
```

### Вход

```bash
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

### Создание поста (с токеном)

```bash
curl -X POST http://localhost:8081/api/v1/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{"title":"Hello","content":"My first post"}'
```
