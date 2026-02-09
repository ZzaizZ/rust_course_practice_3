use crate::domain::entities::errors::DomainResult;
use crate::domain::entities::post::Post;
use crate::domain::entities::user::User;
use crate::domain::repositories::repo::UserRepository;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{debug, error, instrument};
use uuid::Uuid;

#[instrument(skip(connection_string))]
async fn create_pool(connection_string: &str) -> Result<PgPool, sqlx::Error> {
    debug!("Creating database connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(connection_string)
        .await
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            e
        })?;

    debug!("Database connection pool created successfully");
    Ok(pool)
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    #[instrument(skip(connection_string))]
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = create_pool(connection_string).await?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    #[instrument(skip(self, user), fields(username = %user.username, user_id = %user.id))]
    async fn create_user(&self, user: User) -> DomainResult<User> {
        debug!("Inserting user into database");

        let result = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, password_hash, created_at
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.created_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while creating user: {}", e);
            e
        })?;

        debug!("User inserted into database successfully");
        Ok(result)
    }

    #[instrument(skip(self), fields(username = %username))]
    async fn find_by_username(&self, username: &str) -> DomainResult<Option<User>> {
        debug!("Querying user by username");

        let result = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1 OR email = $1;
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while finding user: {}", e);
            e
        })?;

        if result.is_some() {
            debug!("User found in database");
        } else {
            debug!("User not found in database");
        }

        Ok(result)
    }

    #[instrument(skip(self), fields(username = %username))]
    async fn exists_by_username(&self, username: &str) -> DomainResult<bool> {
        debug!("Checking if user exists");

        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM users
                WHERE username = $1 OR email = $1
            ) AS "exists!"
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while checking user existence: {}", e);
            e
        })?;

        debug!("User existence check result: {}", exists);
        Ok(exists)
    }

    #[instrument(skip(self))]
    async fn get_posts(&self) -> DomainResult<Vec<Post>> {
        debug!("Fetching all posts from database");

        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT id AS uuid, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while fetching posts: {}", e);
            e
        })?;

        debug!("Fetched {} posts from database", posts.len());
        Ok(posts)
    }

    #[instrument(skip(self), fields(post_id = %post_id))]
    async fn get_post_by_id(&self, post_id: Uuid) -> DomainResult<Post> {
        debug!("Fetching post by id from database");

        let post = sqlx::query_as!(
            Post,
            r#"
            SELECT id AS uuid, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
            post_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while fetching post: {}", e);
            e
        })?;

        debug!("Post fetched from database successfully");
        Ok(post)
    }

    #[instrument(skip(self, post), fields(post_id = %post.uuid, title = %post.title))]
    async fn create_post(&self, post: Post) -> DomainResult<Post> {
        debug!("Inserting post into database");

        let result = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (id, title, content, author_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING id AS uuid, title, content, author_id, created_at, updated_at
            "#,
            post.uuid,
            post.title,
            post.content,
            post.author_id,
            post.created_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while creating post: {}", e);
            e
        })?;

        debug!("Post inserted into database successfully");
        Ok(result)
    }

    #[instrument(skip(self, post), fields(post_id = %post.uuid))]
    async fn update_post(&self, post: Post) -> DomainResult<Post> {
        debug!("Updating post in database");

        let result = sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET title = $1, content = $2, updated_at = $3
            WHERE id = $4
            RETURNING id AS uuid, title, content, author_id, created_at, updated_at
            "#,
            post.title,
            post.content,
            chrono::Utc::now(),
            post.uuid
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while updating post: {}", e);
            e
        })?;

        debug!("Post updated in database successfully");
        Ok(result)
    }

    #[instrument(skip(self), fields(post_id = %post_id))]
    async fn delete_post(&self, post_id: Uuid) -> DomainResult<()> {
        debug!("Deleting post from database");

        sqlx::query!(
            r#"
            DELETE FROM posts
            WHERE id = $1
            "#,
            post_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while deleting post: {}", e);
            e
        })?;

        debug!("Post deleted from database successfully");
        Ok(())
    }
}
