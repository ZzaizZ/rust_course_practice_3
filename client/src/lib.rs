//! # Client
//!
//! Универсальная клиентская библиотека для взаимодействия с блог-платформой.
//!
//! ## Возможности
//!
//! - HTTP клиент через REST API (с использованием reqwest)
//! - gRPC клиент через tonic
//! - Поддержка WebAssembly (WASM) для использования в браузере
//! - Автоматическое управление JWT токенами
//! - Единый интерфейс [`blog_client::BlogClient`] для всех транспортов
//!
//! ## Features
//!
//! - `http` - Включает HTTP клиент (reqwest)
//! - `grpc` - Включает gRPC клиент (tonic)
//! - `wasm` - Включает поддержку WebAssembly
//! - `default` - Включает и `http`, и `grpc`
//!
//! ## Примеры использования
//!
//! ### HTTP клиент
//!
//! ```rust,no_run
//! use client::{blog_client::BlogClient, http_client::HttpClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = HttpClient::new("http://localhost:8080".to_string()).await?;
//!     
//!     // Регистрация
//!     client.register("user", "user@example.com", "password").await?;
//!     
//!     // Вход
//!     let user_id = client.login("user", "password").await?;
//!     
//!     // Создание поста
//!     let post_id = client.create_post("Title", "Content").await?;
//!     
//!     // Получение списка постов
//!     let posts = client.list_posts(10, 0).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### gRPC клиент
//!
//! ```rust,no_run
//! use client::{blog_client::BlogClient, grpc_client::GrpcClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = GrpcClient::new("http://localhost:50051".to_string()).await?;
//!     
//!     let posts = client.list_posts(10, 0).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod blog_client;
pub mod error;
pub mod types;

#[cfg(feature = "grpc")]
pub mod grpc_client;

#[cfg(feature = "http")]
pub mod http_client;

mod interceptor;
mod token_manager;

// Экспортируем TokenUpdateEvent для использования в WASM-слое
pub use token_manager::TokenUpdateEvent;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
