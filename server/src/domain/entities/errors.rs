use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User already exists: {username}")]
    UserAlreadyExists { username: String },

    #[error("User not found: {username}")]
    UserNotFound { username: String },

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid password: {reason}")]
    InvalidPassword { reason: String },

    #[error("Post not found: {post_id}")]
    PostNotFound { post_id: Uuid },

    #[error("Forbidden: {reason}")]
    Forbidden { reason: String },

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),

    #[error("Token validation failed: {0}")]
    TokenValidationError(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
