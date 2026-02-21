# CLI

Консольный клиент для взаимодействия с блог-платформой.

## Запуск

```bash
# Из корня проекта
cargo run --bin cli -- <COMMAND> [OPTIONS]

# Или из директории cli/
cd cli
cargo run -- <COMMAND> [OPTIONS]
```

## Команды

### Аутентификация

**Регистрация:**

```bash
cargo run --bin cli -- register -u <USERNAME> -e <EMAIL> -p <PASSWORD>

# Пример:
cargo run --bin cli -- register -u alice -e alice@example.com -p password123
```

**Вход:**

```bash
# С паролем в аргументах:
cargo run --bin cli -- login -u <USERNAME> -p <PASSWORD>

# С безопасным вводом пароля с консоли (пароль не отображается):
cargo run --bin cli -- login -u <USERNAME>

# Примеры:
cargo run --bin cli -- login -u alice -p password123
cargo run --bin cli -- login -u alice  # Запросит пароль с консоли
```

После успешного входа токен сохраняется в файл `.blog_token` в текущей директории.

### Управление постами

**Создать пост:**

```bash
cargo run --bin cli -- create-post -t <TITLE> -c <CONTENT>

# Пример:
cargo run --bin cli -- create-post -t "My First Post" -c "Hello, world!"
```

**Получить пост:**

```bash
cargo run --bin cli -- get-post -u <UUID>

# Пример:
cargo run --bin cli -- get-post -u 550e8400-e29b-41d4-a716-446655440000
```

**Обновить пост:**

```bash
cargo run --bin cli -- update-post -u <UUID> -t <TITLE> -c <CONTENT>

# Пример:
cargo run --bin cli -- update-post -u 550e8400-e29b-41d4-a716-446655440000 -t "Updated Title" -c "New content"
```

**Удалить пост:**

```bash
cargo run --bin cli -- delete-post -u <UUID>

# Пример:
cargo run --bin cli -- delete-post -u 550e8400-e29b-41d4-a716-446655440000
```

**Список постов:**

```bash
cargo run --bin cli -- list-posts --page-size <PAGE_SIZE> --page <PAGE>

# Пример (10 постов на странице 0):
cargo run --bin cli -- list-posts --page-size 10 --page 0
```

## Опции

### Выбор транспорта

По умолчанию используется HTTP. Для использования gRPC:

```bash
cargo run --bin cli -- --use-grpc login -u alice -p password123
```

### Указание адреса сервера

По умолчанию `http://localhost:8080`. Для изменения:

```bash
# HTTP сервер
cargo run --bin cli -- --server http://localhost:8081 login -u alice -p password123

# gRPC сервер
cargo run --bin cli -- --use-grpc --server http://localhost:50051 login -u alice
```

## Хранение токенов

JWT токены сохраняются в файл `.blog_token` в **текущей директории** запуска CLI.

- **После login**: Токен автоматически сохраняется в `.blog_token`
- **Для защищённых команд**: Токен автоматически загружается из `.blog_token`
- **При ошибке**: Если токен не найден, CLI выведет ошибку "Failed to read token from file"

**Внимание:** Файл `.blog_token` содержит JWT токен в открытом виде. Храните его в безопасности!

## Примеры использования

### Полный workflow

```bash
# 1. Регистрация
cargo run --bin cli -- register -u alice -e alice@example.com -p password123

# 2. Вход (токен сохраняется в .blog_token)
# Вариант 1: с паролем в аргументах
cargo run --bin cli -- login -u alice -p password123
# Вариант 2: с безопасным вводом пароля (рекомендуется)
cargo run --bin cli -- login -u alice

# 3. Создание поста (использует токен из .blog_token)
cargo run --bin cli -- create-post -t "Hello World" -c "This is my first post"

# 4. Список постов
cargo run --bin cli -- list-posts --page-size 10 --page 0

# 5. Получить конкретный пост
cargo run --bin cli -- get-post -u <UUID_ИЗ_СПИСКА>

# 6. Обновить пост
cargo run --bin cli -- update-post -u <UUID> -t "Updated Title" -c "Updated content"

# 7. Удалить пост
cargo run --bin cli -- delete-post -u <UUID>
```

### Использование gRPC

```bash
# Вход через gRPC с безопасным вводом пароля
cargo run --bin cli -- --use-grpc --server http://localhost:50051 login -u alice

# Создание поста через gRPC
cargo run --bin cli -- --use-grpc --server http://localhost:50051 create-post -t "Test" -c "Content"
```

## Справка

```bash
cargo run --bin cli -- --help
```

Справка по конкретной команде:

```bash
cargo run --bin cli -- <COMMAND> --help
```
