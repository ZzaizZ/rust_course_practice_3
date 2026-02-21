use uuid::Uuid;

use crate::domain::entities::{errors::DomainResult, post::Post, user::User};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: User) -> DomainResult<User>;
    async fn find_by_username(&self, username: &str) -> DomainResult<Option<User>>;
    async fn exists_by_username(&self, username: &str) -> DomainResult<bool>;

    async fn get_posts(&self, page: u32, page_size: u32) -> DomainResult<Vec<Post>>;
    async fn get_post_by_id(&self, post_id: Uuid) -> DomainResult<Post>;
    async fn create_post(&self, post: Post) -> DomainResult<Post>;
    async fn update_post(&self, post: Post) -> DomainResult<Post>;
    async fn delete_post(&self, post_id: Uuid) -> DomainResult<()>;
}
