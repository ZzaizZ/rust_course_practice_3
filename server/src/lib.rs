//! # Server
//!
//! Серверная часть блог-платформы, реализующая Clean Architecture.
//!
//! ## Структура
//!
//! * [`domain`] - Доменная логика (сущности, репозитории, сервисы)
//! * [`application`] - Use cases и DTO для бизнес-логики
//! * [`data`] - Реализация репозиториев (PostgreSQL)
//! * [`infrastructure`] - Конфигурация и внешние зависимости
//! * [`presentation`] - HTTP (REST) и gRPC обработчики
//!
//! ## Основные возможности
//!
//! - Аутентификация через JWT токены
//! - Управление пользователями (регистрация, вход)
//! - CRUD операции с постами блога
//! - REST API (actix-web)
//! - gRPC API (tonic)

pub mod application;
pub mod data;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
