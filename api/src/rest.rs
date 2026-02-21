use serde::{Deserialize, Serialize};

/// Запрос на регистрацию нового пользователя.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Уникальное имя пользователя
    pub username: String,
    /// Пароль (будет захэширован на сервере)
    pub password: String,
    /// Уникальный email адрес
    pub email: String,
}

/// Запрос на вход пользователя.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Имя пользователя или email
    pub username: String,
    /// Пароль
    pub password: String,
}

/// Запрос на обновление access токена с помощью refresh токена.
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    /// JWT refresh токен
    pub refresh_token: String,
}

/// Ответ с JWT токенами.
///
/// Возвращается при успешной аутентификации или обновлении токена.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    /// JWT access токен для аутентификации запросов
    pub access_token: String,
    /// JWT refresh токен для получения нового access токена
    pub refresh_token: String,
    /// Время жизни access токена в секундах
    pub expires_in: i64,
}

/// Запрос на создание нового поста.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    /// Заголовок поста
    pub title: String,
    /// Содержимое поста
    pub content: String,
}

/// Запрос на обновление существующего поста.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    /// Новый заголовок поста
    pub title: String,
    /// Новое содержимое поста
    pub content: String,
}

/// Ответ с данными поста.
#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    /// UUID поста
    pub uuid: String,
    /// Заголовок поста
    pub title: String,
    /// Содержимое поста
    pub content: String,
    /// UUID автора поста
    pub author_id: String,
    /// Временная метка создания (ISO 8601)
    pub created_at: String,
    /// Временная метка последнего обновления (ISO 8601)
    pub updated_at: String,
}
