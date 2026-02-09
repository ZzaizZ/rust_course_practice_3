use std::sync::Arc;
use tonic::{Request, Status};
use crate::domain::services::auth::{AuthService, Claims};

/// Извлекает JWT токен из metadata запроса
pub fn extract_token_from_metadata<T>(request: &Request<T>) -> Result<String, Status> {
    let metadata = request.metadata();
    
    // Пытаемся получить токен из заголовка Authorization
    let auth_header = metadata
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;
    
    let auth_str = auth_header
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;
    
    // Ожидаем формат "Bearer <token>"
    if !auth_str.starts_with("Bearer ") {
        return Err(Status::unauthenticated("Invalid authorization format"));
    }
    
    let token = auth_str.trim_start_matches("Bearer ").to_string();
    Ok(token)
}

/// Interceptor для проверки JWT токена
#[derive(Clone)]
pub struct AuthInterceptor {
    auth_service: Arc<AuthService>,
}

impl AuthInterceptor {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }
    
    /// Проверяет JWT токен и возвращает Claims
    pub fn verify_token<T>(&self, request: &Request<T>) -> Result<Claims, Status> {
        let token = extract_token_from_metadata(request)?;
        
        self.auth_service
            .verify_token(&token)
            .ok_or_else(|| Status::unauthenticated("Invalid or expired token"))
    }
}
