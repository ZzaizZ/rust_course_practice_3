# Blog Platform

Полнофункциональная блог-платформа на Rust с поддержкой HTTP и gRPC API.

## Архитектура

Проект использует Cargo workspace и состоит из следующих крейтов:

- **server** - Веб-сервер с HTTP (REST) и gRPC API
- **api** - Структуры REST-API и protobuf схемы для HTTP и gRPC
- **client** - Универсальная клиентская библиотека-обёртка над HTTP и gRPC
- **cli** - Консольный клиент для взаимодействия с сервером
- **wasm** - Веб-интерфейс на основе Dioxus

## Быстрый старт

### Требования

- Rust 1.75+
- PostgreSQL 14+
- Dioxus CLI (для WASM): `cargo install dioxus-cli`

### Установка

```bash
# Клонировать репозиторий
git clone <repository-url>
cd 3_practice

# (опционально) Создать БД можно через docker-контейнер:
POSTGRES_USER=pguser POSTGRES_PASSWORD=password POSTGRES_DB=pgdb sudo docker-compose up -d

# Настроить переменные окружения внутри сервера
cp server/.env.example server/.env
# Отредактировать .env с параметрами подключения к БД
# Это потребуется для sqlx

# Прописать конфигурацию
cp server/config.yaml.example server/config.yaml

# Собрать проект
cargo build --workspace
```

### Запуск

```bash
# 1. Запустить сервер (HTTP на :8080, gRPC на :50051)
cargo run --bin server -- -c server/config.yaml

# 2. Использовать CLI
cargo run --bin cli -- register -u alice -e alice@example.com -p password123
cargo run --bin cli -- login -u alice  # Запросит пароль с консоли (рекомендуется)
# Или: cargo run --bin cli -- login -u alice -p password123
cargo run --bin cli -- create-post -t "Hello" -c "My first post"
cargo run --bin cli -- list-posts --page-size 10 --page 0

# 3. Запустить WASM фронтенд
cd wasm
BACKEND_URL=http://localhost:8080 dx serve --port 3000
# Откройте в браузере http://localhost:3000
```

## Функциональность

### Аутентификация

- Регистрация пользователей с хешированием паролей (Argon2)
- JWT токены с автоматическим обновлением
- Refresh токены для длительных сессий

### Управление постами

- Создание, чтение, обновление, удаление постов
- Список постов с пагинацией
- Проверка прав доступа (только автор может редактировать)

### Транспорты

- REST API через Actix-web
- gRPC через Tonic
- Единый интерфейс в клиентской библиотеке

## Разработка

```bash
# Запустить все тесты
cargo test --workspace

# Проверить код
cargo clippy --workspace

# Форматирование
cargo fmt --workspace
```

## Структура БД

```sql
-- users: id, username, email, password_hash, created_at
-- posts: id, title, content, author_id, created_at, updated_at
```

Миграции находятся в `server/migrations/`.

## Документация

Подробная документация для каждого компонента находится в соответствующих директориях:

- [Server README](server/README.md)
- [Client README](client/README.md)
- [CLI README](cli/README.md)
- [WASM README](wasm/README.md)
- [API README](api/README.md)
