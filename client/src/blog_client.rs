use crate::types;
use async_trait::async_trait;
use uuid::Uuid;

pub enum Transport {
    Grpc(String),
    Http(String),
}

#[async_trait]
pub trait BlogClient {
    async fn login(&self, username: &str, password: &str) -> types::ClientResult<Uuid>;
    async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> types::ClientResult<()>;
    async fn setup_token(&self, token: &str) -> types::ClientResult<()>;
    async fn get_token(&self) -> types::ClientResult<Option<String>>;

    async fn create_post(&self, title: &str, content: &str) -> types::ClientResult<Uuid>;
    async fn get_post(&self, post_id: &str) -> types::ClientResult<types::Post>;
    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> types::ClientResult<()>;
    async fn delete_post(&self, post_id: &str) -> types::ClientResult<()>;
    async fn list_posts(&self, page_size: u8, page: u32) -> types::ClientResult<Vec<types::Post>>;
}
