use crate::application::dto::post::{CreatePostDto, PostDto, UpdatePostDto};
use crate::domain::entities::errors::DomainResult;
use crate::domain::entities::post::Post;
use crate::domain::repositories::repo::UserRepository;
use std::sync::Arc;
use tracing::{info, debug, instrument};
use uuid::Uuid;

pub struct PostApplication<Repo: UserRepository> {
    user_repository: Arc<Repo>,
}

impl<Repo: UserRepository> PostApplication<Repo> {
    pub fn new(user_repository: Arc<Repo>) -> Self {
        Self { user_repository }
    }

    #[instrument(skip(self))]
    pub async fn get_all_posts(&self) -> DomainResult<Vec<PostDto>> {
        debug!("Fetching all posts");
        let posts = self.user_repository.get_posts().await?;
        info!("Retrieved {} posts", posts.len());
        Ok(posts.into_iter().map(PostDto::from_entity).collect())
    }

    #[instrument(skip(self), fields(post_id = %post_id))]
    pub async fn get_post_by_id(&self, post_id: Uuid) -> DomainResult<PostDto> {
        debug!("Fetching post by id");
        let post = self.user_repository.get_post_by_id(post_id).await?;
        info!("Post retrieved successfully");
        Ok(PostDto::from_entity(post))
    }

    #[instrument(skip(self, dto), fields(title = %dto.title, author_id = %dto.author_id))]
    pub async fn create_post(&self, dto: CreatePostDto) -> DomainResult<PostDto> {
        debug!("Creating new post");
        
        let now = chrono::Utc::now();
        let post = Post {
            uuid: Uuid::now_v7(),
            title: dto.title,
            content: dto.content,
            author_id: dto.author_id,
            created_at: now,
            updated_at: now,
        };

        let created_post = self.user_repository.create_post(post).await?;
        info!("Post created successfully with id: {}", created_post.uuid);
        Ok(PostDto::from_entity(created_post))
    }

    #[instrument(skip(self, dto), fields(post_id = %dto.uuid, title = %dto.title))]
    pub async fn update_post(&self, dto: UpdatePostDto) -> DomainResult<PostDto> {
        debug!("Updating post");
        
        // Проверяем, существует ли пост
        let existing_post = self.user_repository.get_post_by_id(dto.uuid).await?;

        let updated_post = Post {
            uuid: dto.uuid,
            title: dto.title,
            content: dto.content,
            author_id: existing_post.author_id,
            created_at: existing_post.created_at,
            updated_at: chrono::Utc::now(),
        };

        let result = self.user_repository.update_post(updated_post).await?;
        info!("Post updated successfully");
        Ok(PostDto::from_entity(result))
    }

    #[instrument(skip(self), fields(post_id = %post_id))]
    pub async fn delete_post(&self, post_id: Uuid) -> DomainResult<()> {
        debug!("Deleting post");
        self.user_repository.delete_post(post_id).await?;
        info!("Post deleted successfully");
        Ok(())
    }
}
