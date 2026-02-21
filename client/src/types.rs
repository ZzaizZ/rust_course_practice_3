use uuid::Uuid;

/// Тип Result для операций клиента.
pub type ClientResult<T> = Result<T, crate::error::ClientError>;

/// Представление поста блога.
///
/// Содержит все данные поста, включая метаданные о создании и обновлении.
#[derive(Debug, Clone)]
pub struct Post {
    /// Уникальный идентификатор поста
    pub id: Uuid,
    /// Заголовок поста
    pub title: String,
    /// Содержимое поста
    pub content: String,
    /// Временная метка создания
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Временная метка последнего обновления
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Представление пользователя.
#[derive(Debug, Clone)]
pub struct User {
    /// Уникальный идентификатор пользователя
    pub id: Uuid,
    /// Имя пользователя
    pub username: String,
}

pub(crate) type Token = String;

/// Данные аутентификации (внутренний тип).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthData {
    /// JWT access токен
    pub access_token: Token,
    /// JWT refresh токен
    pub refresh_token: Token,
}
