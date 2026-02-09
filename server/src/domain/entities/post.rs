use uuid::Uuid;

pub struct Post {
    pub uuid: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
