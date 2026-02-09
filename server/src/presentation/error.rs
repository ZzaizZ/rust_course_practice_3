use crate::domain::entities::errors::DomainError;
use actix_web::HttpResponse;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl ApiError {
    pub fn bad_request(message: String) -> Self {
        Self::BadRequest(message)
    }

    pub fn unauthorized(message: String) -> Self {
        Self::Unauthorized(message)
    }

    pub fn forbidden(message: String) -> Self {
        Self::Forbidden(message)
    }

    pub fn not_found(message: String) -> Self {
        Self::NotFound(message)
    }

    pub fn internal_server_error(message: String) -> Self {
        Self::InternalServerError(message)
    }
}

impl actix_web::error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        error!("API Error: {}", self);
        let status = self.status_code();
        HttpResponse::build(status).json(serde_json::json!({
            "error": self.to_string()
        }))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => actix_web::http::StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            ApiError::InternalServerError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::UserAlreadyExists { .. } => Self::bad_request(err.to_string()),
            DomainError::UserNotFound { .. } => Self::not_found(err.to_string()),
            DomainError::InvalidCredentials => Self::unauthorized(err.to_string()),
            DomainError::InvalidPassword { .. } => Self::bad_request(err.to_string()),
            DomainError::PostNotFound { .. } => Self::not_found(err.to_string()),
            DomainError::Forbidden { .. } => Self::forbidden(err.to_string()),
            DomainError::RepositoryError(_) => Self::internal_server_error(err.to_string()),
            DomainError::TokenGenerationError(_) => Self::internal_server_error(err.to_string()),
            DomainError::TokenValidationError(_) => Self::unauthorized(err.to_string()),
        }
    }
}
