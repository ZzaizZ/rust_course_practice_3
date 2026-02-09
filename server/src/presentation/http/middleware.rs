use actix_web::{HttpMessage, dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::domain::services::auth::AuthService;

/// Структура для хранения информации об аутентифицированном пользователе
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
}

/// Валидатор JWT токена для actix-web-httpauth middleware
///
/// Эта функция извлекает токен из заголовка Authorization,
/// проверяет его через AuthService и добавляет информацию о пользователе
/// в расширения запроса для использования в хэндлерах.
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let token = credentials.token();
    debug!("Validating JWT token");

    // Получаем AuthService из app_data
    let auth_service = req
        .app_data::<web::Data<AuthService>>()
        .map(|data| data.get_ref());

    if auth_service.is_none() {
        warn!("AuthService not found in app_data");
        return Err((ErrorUnauthorized("Internal server error"), req));
    }

    let auth_service = auth_service.unwrap();

    // Проверяем токен
    match auth_service.verify_token(token) {
        Some(claims) => {
            debug!(
                "Token validated successfully for user: {}",
                claims.user_name
            );

            // Парсим user_id из claims.sub
            let user_id = match Uuid::parse_str(&claims.sub) {
                Ok(id) => id,
                Err(_) => {
                    warn!("Invalid UUID in token claims: {}", claims.sub);
                    return Err((ErrorUnauthorized("Invalid token format"), req));
                }
            };

            // Создаём структуру аутентифицированного пользователя
            let authenticated_user = AuthenticatedUser {
                user_id,
                username: claims.user_name,
            };

            // Добавляем информацию о пользователе в расширения запроса
            req.extensions_mut().insert(authenticated_user);

            Ok(req)
        }
        None => {
            warn!("Token validation failed");
            Err((ErrorUnauthorized("Invalid or expired token"), req))
        }
    }
}
