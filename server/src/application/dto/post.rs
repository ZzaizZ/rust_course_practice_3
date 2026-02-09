use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreatePostDto {
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdatePostDto {
    pub uuid: Uuid,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct PostDto {
    pub uuid: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PostDto {
    pub fn from_entity(post: crate::domain::entities::post::Post) -> Self {
        Self {
            uuid: post.uuid,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}
