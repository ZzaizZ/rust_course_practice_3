# Client

Универсальная клиентская библиотека для взаимодействия с блог-платформой.

## Возможности

- HTTP клиент через REST API (reqwest)
- gRPC клиент через Tonic
- Поддержка WebAssembly (WASM)
- Автоматическое управление JWT токенами
- Единый интерфейс `BlogClient` для всех транспортов
- Автоматическое обновление истекающих токенов

## Features

- `http` - HTTP клиент (включен по умолчанию)
- `grpc` - gRPC клиент (включен по умолчанию)
- `wasm` - Поддержка WebAssembly
- `default` - Включает и `http`, и `grpc`

## Использование

### HTTP клиент

```rust
use client::http_client::HttpClient;
use client::blog_client::BlogClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HttpClient::new("http://localhost:8081".to_string()).await?;
    
    // Регистрация
    client.register("alice", "alice@example.com", "password123").await?;
    
    // Вход
    let user_id = client.login("alice", "password123").await?;
    
    // Создание поста
    let post_id = client.create_post("My Title", "Post content").await?;
    
    // Получение поста
    let post = client.get_post(&post_id.to_string()).await?;
    
    // Список постов (page_size, page)
    let posts = client.list_posts(10, 0).await?;
    
    Ok(())
}
```

### gRPC клиент

```rust
use client::grpc_client::GrpcClient;
use client::blog_client::BlogClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GrpcClient::new("http://localhost:50051").await?;
    
    // Тот же интерфейс, что и HTTP клиент
    client.register("alice", "alice@example.com", "password123").await?;
    client.login("alice", "password123").await?;
    
    let posts = client.list_posts(10, 0).await?;
    
    Ok(())
}
```

### Работа с токенами

```rust
// Токены сохраняются автоматически после login/register
client.login("alice", "password123").await?;

// Получить текущий токен
if let Ok(Some(token)) = client.get_token().await {
    println!("Access token: {}", token);
}

// Установить токен вручную
client.setup_token("your_jwt_token_here").await?;

// Токены автоматически обновляются при необходимости
// (за 5 минут до истечения по умолчанию)
```

### WASM

Для использования в WebAssembly:

```toml
[dependencies]
client = { path = "../client", default-features = false, features = ["http"] }
```

```rust
// WASM пример
use client::http_client::HttpClient;

// В WASM HttpClient работает через браузерный fetch API
let client = HttpClient::new("http://localhost:8081".to_string()).await?;
```

## API

### Трейт BlogClient

Все клиенты реализуют единый трейт:

```rust
#[async_trait]
pub trait BlogClient {
    // Аутентификация
    async fn login(&self, username: &str, password: &str) -> ClientResult<Uuid>;
    async fn register(&self, username: &str, email: &str, password: &str) -> ClientResult<()>;
    async fn setup_token(&self, token: &str) -> ClientResult<()>;
    async fn get_token(&self) -> ClientResult<Option<String>>;
    
    // Посты
    async fn create_post(&self, title: &str, content: &str) -> ClientResult<Uuid>;
    async fn get_post(&self, post_id: &str) -> ClientResult<Post>;
    async fn update_post(&self, post_id: &str, title: &str, content: &str) -> ClientResult<()>;
    async fn delete_post(&self, post_id: &str) -> ClientResult<()>;
    async fn list_posts(&self, page_size: u8, page: u32) -> ClientResult<Vec<Post>>;
}
```

## Хранение токенов

Библиотека автоматически управляет токенами:

- **HTTP/gRPC клиенты**: Токены хранятся в памяти в `TokenManager`
- **CLI**: Токены сохраняются в файл `.blog_token` в текущей директории
- **WASM**: Токены сохраняются в `localStorage` браузера

Refresh токены используются для автоматического обновления access токенов.

## Обработка ошибок

Все методы возвращают `ClientResult<T>`:

```rust
pub type ClientResult<T> = Result<T, ClientError>;

pub enum ClientError {
    TransportError(String),      // Ошибки сети/соединения
    AuthError(String),            // Ошибки аутентификации
    NotFound(String),             // Ресурс не найден
    ValidationError(String),      // Ошибки валидации
    ServerError(String),          // Ошибки сервера
}
```

## Примеры

См. `examples/` в директории библиотеки или используйте CLI/WASM проекты как справочник.
