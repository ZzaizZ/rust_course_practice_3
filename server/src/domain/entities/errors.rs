use thiserror::Error;
use uuid::Uuid;

/// Ошибки доменного слоя.
///
/// Представляет все возможные ошибки, которые могут возникнуть
/// при выполнении бизнес-логики приложения.
#[derive(Error, Debug)]
pub enum DomainError {
    /// Пользователь с таким именем уже существует
    #[error("User already exists: {username}")]
    UserAlreadyExists { username: String },

    /// Пользователь не найден
    #[error("User not found: {username}")]
    UserNotFound { username: String },

    /// Неверные учётные данные (пароль или логин)
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Невалидный пароль (не соответствует требованиям)
    #[error("Invalid password: {reason}")]
    InvalidPassword { reason: String },

    /// Пост не найден
    #[error("Post not found: {post_id}")]
    PostNotFound { post_id: Uuid },

    /// Запрещённое действие (например, редактирование чужого поста)
    #[error("Forbidden: {reason}")]
    Forbidden { reason: String },

    /// Ошибка на уровне репозитория (БД)
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// Ошибка при генерации токена
    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),

    /// Ошибка при валидации токена
    #[error("Token validation failed: {0}")]
    TokenValidationError(String),
}

/// Тип Result для операций доменного слоя.
pub type DomainResult<T> = Result<T, DomainError>;
