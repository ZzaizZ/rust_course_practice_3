use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::ClientError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_name: String,
    pub exp: i64,
}

impl Claims {
    /// Проверяет, истечет ли токен в ближайшее время (в течение buffer_seconds)
    pub fn expires_soon(&self, buffer_seconds: i64) -> bool {
        let now = Utc::now().timestamp();
        self.exp <= now + buffer_seconds
    }
}

/// Декодирует JWT токен без проверки подписи
/// (для клиента проверка подписи не требуется, так как мы получаем токен от доверенного сервера)
pub fn decode_token_without_validation(token: &str) -> Result<Claims, ClientError> {
    let token_data = jsonwebtoken::dangerous::insecure_decode::<Claims>(token)
        .map_err(|e| ClientError::InternalError(format!("Failed to decode token: {}", e)))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expires_soon() {
        let claims = Claims {
            sub: "user_id".to_string(),
            user_name: "test_user".to_string(),
            exp: Utc::now().timestamp() + 30, // истечет через 30 секунд
        };
        assert!(claims.expires_soon(60)); // Истекает в течение минуты
        assert!(!claims.expires_soon(10)); // Не истекает в течение 10 секунд
    }
}
