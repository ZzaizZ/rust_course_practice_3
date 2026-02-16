use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use api::rest::{
    CreatePostRequest, LoginRequest, PostResponse, RefreshTokenRequest, RegisterRequest,
    TokenResponse, UpdatePostRequest,
};

use crate::application::auth::AuthApplication;
use crate::application::dto::auth::{LoginDto, RegisterDto, TokenDto};
use crate::application::dto::post::{CreatePostDto, PostDto, UpdatePostDto};
use crate::application::post::PostApplication;
use crate::data::pgrepo::PgUserRepository;
use crate::presentation::error::ApiError;
use crate::presentation::http::middleware::AuthenticatedUser;

// Структура для хранения зависимостей приложения
pub struct AppState {
    pub auth_app: Arc<AuthApplication<PgUserRepository>>,
    pub post_app: Arc<PostApplication<PgUserRepository>>,
}

impl From<TokenDto> for TokenResponse {
    fn from(dto: TokenDto) -> Self {
        Self {
            access_token: dto.access_token,
            refresh_token: dto.refresh_token,
            expires_in: dto.expires_in,
        }
    }
}

impl From<PostDto> for PostResponse {
    fn from(dto: PostDto) -> Self {
        Self {
            uuid: dto.uuid.to_string(),
            title: dto.title,
            content: dto.content,
            author_id: dto.author_id.to_string(),
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}

#[post("/api/v1/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> Result<impl Responder, ApiError> {
    info!(
        "Received registration request for username: {}",
        req.username
    );

    let dto = RegisterDto {
        username: req.username.clone(),
        password: req.password.clone(),
        email: req.email.clone(),
    };

    let user = state.auth_app.create_user(dto).await?;

    info!("User registered successfully: {}", user.username);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": user.id.to_string(),
        "username": user.username,
        "email": user.email,
        "created_at": user.created_at.to_rfc3339()
    })))
}

#[post("/api/v1/auth/login")]
pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<impl Responder, ApiError> {
    info!("Received login request for username: {}", req.username);

    let dto = LoginDto {
        username: req.username.clone(),
        password: req.password.clone(),
    };

    let token_dto = state.auth_app.login(dto).await?;
    let response = TokenResponse::from(token_dto);

    info!("User logged in successfully: {}", req.username);

    Ok(HttpResponse::Ok().json(response))
}

#[post("/api/v1/auth/refresh")]
pub async fn refresh_token(
    state: web::Data<AppState>,
    req: web::Json<RefreshTokenRequest>,
) -> Result<impl Responder, ApiError> {
    info!("Received token refresh request");

    let token_dto = state
        .auth_app
        .refresh_token(req.refresh_token.clone())
        .await?;
    let response = TokenResponse::from(token_dto);

    info!("Token refreshed successfully");

    Ok(HttpResponse::Ok().json(response))
}

#[post("/api/v1/posts")]
pub async fn create_post(
    http_req: HttpRequest,
    state: web::Data<AppState>,
    req: web::Json<CreatePostRequest>,
) -> Result<impl Responder, ApiError> {
    info!("Received request to create post: {}", req.title);

    // Извлекаем информацию об аутентифицированном пользователе из extensions
    let auth_user = http_req
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| {
            warn!("AuthenticatedUser not found in request extensions");
            ApiError::unauthorized("Authentication required".to_string())
        })?;

    info!(
        "Creating post for user: {} ({})",
        auth_user.username, auth_user.user_id
    );

    let dto = CreatePostDto {
        title: req.title.clone(),
        content: req.content.clone(),
        author_id: auth_user.user_id,
    };

    let post_dto = state.post_app.create_post(dto).await?;
    let response = PostResponse::from(post_dto);

    info!("Post created successfully: {}", req.title);

    Ok(HttpResponse::Created().json(response))
}

#[get("/api/v1/posts")]
pub async fn list_posts(state: web::Data<AppState>) -> Result<impl Responder, ApiError> {
    info!("Received request to list all posts");

    let posts = state.post_app.get_all_posts().await?;
    let response: Vec<PostResponse> = posts.into_iter().map(PostResponse::from).collect();

    info!("Returning {} posts", response.len());

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/v1/posts/{id}")]
pub async fn get_post(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let post_id_str = path.into_inner();
    info!("Received request to get post: {}", post_id_str);

    let post_id = Uuid::parse_str(&post_id_str).map_err(|_| {
        warn!("Invalid UUID format: {}", post_id_str);
        ApiError::bad_request("Invalid UUID format".to_string())
    })?;

    let post_dto = state.post_app.get_post_by_id(post_id).await?;
    let response = PostResponse::from(post_dto);

    info!("Post retrieved successfully: {}", post_id);

    Ok(HttpResponse::Ok().json(response))
}

#[put("/api/v1/posts/{id}")]
pub async fn update_post(
    http_req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: web::Json<UpdatePostRequest>,
) -> Result<impl Responder, ApiError> {
    let post_id_str = path.into_inner();
    info!("Received request to update post: {}", post_id_str);

    // Извлекаем информацию об аутентифицированном пользователе
    let auth_user = http_req
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| {
            warn!("AuthenticatedUser not found in request extensions");
            ApiError::unauthorized("Authentication required".to_string())
        })?;

    let post_id = Uuid::parse_str(&post_id_str).map_err(|_| {
        warn!("Invalid UUID format: {}", post_id_str);
        ApiError::bad_request("Invalid UUID format".to_string())
    })?;

    // Проверяем, что пользователь является автором поста
    let existing_post = state.post_app.get_post_by_id(post_id).await?;
    if existing_post.author_id != auth_user.user_id {
        warn!(
            "User {} attempted to update post {} owned by {}",
            auth_user.user_id, post_id, existing_post.author_id
        );
        return Err(ApiError::forbidden(
            "You can only update your own posts".to_string(),
        ));
    }

    let dto = UpdatePostDto {
        uuid: post_id,
        title: req.title.clone(),
        content: req.content.clone(),
    };

    let post_dto = state.post_app.update_post(dto).await?;
    let response = PostResponse::from(post_dto);

    info!("Post updated successfully: {}", post_id);

    Ok(HttpResponse::Ok().json(response))
}

#[delete("/api/v1/posts/{id}")]
pub async fn delete_post(
    http_req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let post_id_str = path.into_inner();
    info!("Received request to delete post: {}", post_id_str);

    // Извлекаем информацию об аутентифицированном пользователе
    let auth_user = http_req
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| {
            warn!("AuthenticatedUser not found in request extensions");
            ApiError::unauthorized("Authentication required".to_string())
        })?;

    let post_id = Uuid::parse_str(&post_id_str).map_err(|_| {
        warn!("Invalid UUID format: {}", post_id_str);
        ApiError::bad_request("Invalid UUID format".to_string())
    })?;

    // Проверяем, что пользователь является автором поста
    let existing_post = state.post_app.get_post_by_id(post_id).await?;
    if existing_post.author_id != auth_user.user_id {
        warn!(
            "User {} attempted to delete post {} owned by {}",
            auth_user.user_id, post_id, existing_post.author_id
        );
        return Err(ApiError::forbidden(
            "You can only delete your own posts".to_string(),
        ));
    }

    state.post_app.delete_post(post_id).await?;

    info!("Post deleted successfully: {}", post_id);

    Ok(HttpResponse::NoContent().finish())
}
