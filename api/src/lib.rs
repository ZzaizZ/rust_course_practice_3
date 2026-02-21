//! # API
//!
//! Общие типы данных для REST и gRPC API блог-платформы.
//!
//! ## Features
//!
//! * `rest` - Включает REST API типы (JSON serialization)
//! * `grpc` - Включает gRPC типы (Protocol Buffers)
//! * `default` - Включает оба: `rest` и `grpc`
//!
//! ## REST API
//!
//! При включении feature `rest` доступен модуль [`rest`] с типами запросов и ответов
//! для HTTP API (сериализация в JSON через serde).
//!
//! ## gRPC API
//!
//! При включении feature `grpc` типы генерируются автоматически из
//! `proto/blog.proto` с помощью tonic-build. Доступны сервисы и типы
//! для gRPC взаимодействия.

#[cfg(feature = "rest")]
pub mod rest;

#[cfg(feature = "grpc")]
pub mod api {
    tonic::include_proto!("blog");
}

#[cfg(feature = "grpc")]
pub use api::*;
