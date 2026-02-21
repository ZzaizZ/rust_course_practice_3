# WASM

Веб-интерфейс блог-платформы на основе Dioxus framework.

## Требования

- Rust 1.75+
- Dioxus CLI:

  ```bash
  cargo install dioxus-cli
  ```

## Запуск

### Режим разработки

```bash
cd wasm
dx serve
```

По умолчанию приложение будет доступно по адресу: `http://localhost:8080`

#### Настройка порта и адреса

Вы можете кастомизировать порт и адрес запуска с помощью параметров командной строки:

```bash
# Запуск на другом порту
dx serve --port 3000

# Запуск на другом адресе
dx serve --addr 0.0.0.0

# Комбинация параметров
dx serve --port 3000 --addr 0.0.0.0
```

**Параметры:**

- `--port <PORT>` - указать порт (по умолчанию: 8080)
- `--addr <ADDR>` - указать адрес (по умолчанию: 127.0.0.1)
  - Используйте `0.0.0.0` для доступа извне (например, из локальной сети)
- `--open false` - не открывать браузер автоматически

**Примеры:**

```bash
# Запуск на порту 3000 с автоматическим открытием браузера
dx serve --port 3000

# Запуск для внешнего доступа без открытия браузера
dx serve --addr 0.0.0.0 --port 8000 --open false
```

#### Настройка backend URL

По умолчанию приложение подключается к backend по адресу `http://localhost:8081`.

**Способ 1: Через конфигурационный файл (рекомендуется)**

Отредактируйте `Dioxus.toml`:

```toml
[application.env]
BACKEND_URL = "http://your-backend:8081"
```

После изменения просто запустите:

```bash
dx serve
```

**Способ 2: Через переменную окружения**

Переопределите настройку при запуске:

```bash
# Запуск с кастомным backend URL
BACKEND_URL=http://192.168.1.100:8081 dx serve

# Запуск с production backend
BACKEND_URL=https://api.example.com dx serve

# Комбинация всех параметров
BACKEND_URL=http://10.0.0.5:9000 dx serve --port 3000 --addr 0.0.0.0
```

**Важно:** Переменная `BACKEND_URL` встраивается в WASM при компиляции. Приоритет: переменная окружения > значение в Dioxus.toml > дефолтное значение.

### Сборка для продакшена

```bash
cd wasm
dx build --release
```

Для продакшена с кастомным backend:

```bash
BACKEND_URL=https://api.example.com dx build --release
```

Собранные файлы будут в `wasm/dist/`.

## Функциональность

### Аутентификация

- **Регистрация**: Кнопка "Sign Up" в правом верхнем углу
  - Поля: username, email, password
  - После регистрации токен сохраняется автоматически

- **Вход**: Кнопка "Sign In" в правом верхнем углу
  - Поля: username, password
  - После входа токен сохраняется автоматически

- **Выход**: Кнопка "Logout" (доступна после входа)
  - Очищает токен из localStorage

### Посты

- **Список постов**: Отображается на главной странице
  - Доступен всем пользователям (публичный)
  - Автоматически обновляется

- **Создание поста**: Кнопка "New Post" (только для авторизованных)
  - Поля: title, content
  - Пост привязывается к текущему пользователю

- **Редактирование**: Кнопка "Edit" на карточке поста
  - Доступна только автору поста
  - Можно изменить заголовок и содержимое

- **Удаление**: Кнопка "Delete" на карточке поста
  - Доступна только автору поста
  - Требует подтверждения

## Хранение токенов

JWT токены сохраняются в **localStorage** браузера:

- **Ключ**: `blog_auth_token`
- **Значение**: JWT access token
- **Lifetime**: Управляется сервером (обычно 1 час)

Токен автоматически:

- Сохраняется после успешного login/register
- Загружается при открытии страницы
- Удаляется при logout

**Просмотр токена в браузере:**

1. Откройте DevTools (F12)
2. Перейдите в Application → Local Storage
3. Найдите ключ `blog_auth_token`

**Файл для хранения**: `src/storage.rs` предоставляет функции:

- `save_token(token: &str)` - сохранить токен в localStorage
- `load_token() -> Option<String>` - загрузить токен из localStorage
- `clear_token()` - удалить токен из localStorage

## Структура компонентов

- `AuthenticatedApp` - Главный компонент с навигацией
- `LoginForm` - Форма входа
- `RegisterForm` - Форма регистрации
- `PostsList` - Список всех постов
- `PostCard` - Карточка поста
- `PostForm` - Форма создания/редактирования поста
- `PostView` - Просмотр поста

## Конфигурация

### Адрес сервера

По умолчанию: `http://localhost:8081`

Для изменения отредактируйте `src/main.rs`:

```rust
let client = HttpClient::new_with_token_notifier(
    "http://your-server:8081".to_string(),
    token_sender
).await?;
```

### Стилизация

Проект использует Tailwind CSS. Файлы стилей:

- `assets/tailwind.css` - сгенерированный Tailwind CSS
- `assets/styling/main.css` - кастомные стили

Для пересборки Tailwind:

```bash
# Установить Tailwind CLI
npm install -g tailwindcss

# Собрать стили
tailwindcss -o assets/tailwind.css --minify
```

## Использование HTTP-Client из ../client

Трейт `BlogClient` предоставляет полный API:

```rust
#[async_trait(?Send)]  // ?Send для WASM
pub trait BlogClient {
    async fn login(&self, username: &str, password: &str) -> ClientResult<Uuid>;
    async fn register(&self, username: &str, email: &str, password: &str) -> ClientResult<()>;
    async fn setup_token(&self, token: &str) -> ClientResult<()>;
    async fn get_token(&self) -> ClientResult<Option<String>>;
    
    async fn create_post(&self, title: &str, content: &str) -> ClientResult<Uuid>;
    async fn get_post(&self, post_id: &str) -> ClientResult<Post>;
    async fn update_post(&self, post_id: &str, title: &str, content: &str) -> ClientResult<()>;
    async fn delete_post(&self, post_id: &str) -> ClientResult<()>;
    async fn list_posts(&self, page_size: u8, page: u32) -> ClientResult<Vec<Post>>;
}
```

## Разработка

### Hot reload

Dioxus CLI автоматически перезагружает страницу при изменении кода:

```bash
dx serve
```

### Отладка

Откройте консоль браузера (F12) для просмотра логов и ошибок.

### Build profiles

```bash
# Development (быстрая сборка, большой размер)
dx serve

# Release (оптимизированная сборка)
dx build --release
```

## Деплой

### Статический хостинг

После `dx build --release` загрузите содержимое `dist/` на любой статический хостинг:

- GitHub Pages
- Netlify
- Vercel
- AWS S3
- Cloudflare Pages

### Nginx конфигурация

```nginx
server {
    listen 80;
    server_name example.com;
    root /path/to/dist;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

## Как работает WASM-совместимость

Крейт `client` адаптирован для WASM следующим образом:

1. **async-trait с ?Send**: `#[async_trait(?Send)]` для WASM, где нет многопоточности
2. **reqwest без blocking**: используется async версия reqwest
3. **tokio для WASM**: минимальный набор фич - только `sync` и `macros`
4. **Условная компиляция**: через `#[cfg(target_arch = "wasm32")]`

Благодаря этому один и тот же код клиента работает:

- В WASM (браузер)
- В native приложениях (desktop, CLI)
- На сервере

## Известные ограничения

- CORS: Сервер должен разрешать запросы с origin фронтенда
- WebAssembly: Не все браузеры полностью поддерживают WASM
- LocalStorage: Токены хранятся в открытом виде (как в cookies)

## Troubleshooting

**Ошибка CORS:**
Проверьте настройку `cors_origin` в `server/config.yml`:

```yaml
cors_origin: http://localhost:8080
```

**Токен не сохраняется:**
Проверьте консоль браузера на ошибки localStorage. Убедитесь, что функции из `storage.rs` вызываются корректно.

**Не загружаются стили:**
Убедитесь, что файлы в `assets/` скопированы при сборке. Dioxus 0.7+ автоматически обрабатывает Tailwind.

**Проблемы компиляции с tokio:**
Убедитесь что client крейт использует правильные фичи tokio для WASM (только sync и macros, без rt-multi-thread).
