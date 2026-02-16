use uuid::Uuid;

pub type ClientResult<T> = Result<T, crate::error::ClientError>;

pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct User {
    pub id: Uuid,
    pub username: String,
}

pub(crate) type Token = String;

#[derive(Debug, Clone)]
pub(crate) struct AuthData {
    pub access_token: Token,
    pub refresh_token: Token,
}
