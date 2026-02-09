use std::sync::Arc;

use api::blog_server::Blog;
use api::{
    CreatePostRequest, DeletePostRequest, DeletePostResponse, GetPostRequest, JwtContainer,
    ListPostsRequest, ListPostsResponse, LoginRequest, LoginResponse, Post as ProtoPost,
    PostResponse, RegisterRequest, RegisterResponse, Response as ProtoResponse,
    Status as ProtoStatus, UpdatePostRequest,
};
use prost_types::Timestamp;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use super::auth::AuthInterceptor;
use crate::application::auth::AuthApplication;
use crate::application::dto::auth::{LoginDto, RegisterDto};
use crate::application::dto::post::{CreatePostDto, UpdatePostDto};
use crate::application::post::PostApplication;
use crate::domain::entities::errors::DomainError;
use crate::domain::repositories::repo::UserRepository;
use crate::domain::services::auth::AuthService;

pub struct BlogServiceImpl<Repo: UserRepository> {
    auth_app: Arc<AuthApplication<Repo>>,
    post_app: Arc<PostApplication<Repo>>,
    auth_interceptor: AuthInterceptor,
}

impl<Repo: UserRepository> BlogServiceImpl<Repo> {
    pub fn new(
        auth_app: Arc<AuthApplication<Repo>>,
        post_app: Arc<PostApplication<Repo>>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        Self {
            auth_app,
            post_app,
            auth_interceptor: AuthInterceptor::new(auth_service),
        }
    }

    fn map_domain_error(error: DomainError) -> ProtoResponse {
        match error {
            DomainError::UserAlreadyExists { .. } => ProtoResponse {
                code: ProtoStatus::InvalidRequest as i32,
                details: Some(error.to_string()),
            },
            DomainError::UserNotFound { .. } => ProtoResponse {
                code: ProtoStatus::Unauthorized as i32,
                details: Some(error.to_string()),
            },
            DomainError::InvalidCredentials => ProtoResponse {
                code: ProtoStatus::Unauthorized as i32,
                details: Some(error.to_string()),
            },
            DomainError::PostNotFound { .. } => ProtoResponse {
                code: ProtoStatus::InvalidRequest as i32,
                details: Some(error.to_string()),
            },
            DomainError::Forbidden { .. } => ProtoResponse {
                code: ProtoStatus::Unauthorized as i32,
                details: Some(error.to_string()),
            },
            _ => ProtoResponse {
                code: ProtoStatus::InternalError as i32,
                details: Some(error.to_string()),
            },
        }
    }
}

#[tonic::async_trait]
impl<Repo: UserRepository + Send + Sync + 'static> Blog for BlogServiceImpl<Repo> {
    #[instrument(skip(self, request))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        debug!("Register request received for login: {}", req.login);

        let dto = RegisterDto {
            username: req.login,
            email: req.email,
            password: req.password,
        };

        match self.auth_app.create_user(dto).await {
            Ok(_user) => {
                info!("User registered successfully");
                Ok(Response::new(RegisterResponse {
                    status: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("User registered successfully".to_string()),
                    }),
                }))
            }
            Err(e) => {
                warn!("User registration failed: {}", e);
                Ok(Response::new(RegisterResponse {
                    status: Some(Self::map_domain_error(e)),
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        debug!("Login request received for: {}", req.email_or_login);

        let dto = LoginDto {
            username: req.email_or_login,
            password: req.password,
        };

        match self.auth_app.login(dto).await {
            Ok(token_dto) => {
                info!("User logged in successfully");

                let expires_at =
                    chrono::Utc::now() + chrono::Duration::seconds(token_dto.expires_in);

                Ok(Response::new(LoginResponse {
                    status: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("Login successful".to_string()),
                    }),
                    token: Some(JwtContainer {
                        access_token: token_dto.access_token,
                        refresh_token: token_dto.refresh_token,
                        expires_in: Some(Timestamp {
                            seconds: expires_at.timestamp(),
                            nanos: expires_at.timestamp_subsec_nanos() as i32,
                        }),
                    }),
                }))
            }
            Err(e) => {
                warn!("Login failed: {}", e);
                Ok(Response::new(LoginResponse {
                    status: Some(Self::map_domain_error(e)),
                    token: None,
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        // Проверяем JWT токен и извлекаем claims
        let claims = self.auth_interceptor.verify_token(&request)?;
        debug!("Authenticated user: {}", claims.user_name);

        let req = request.into_inner();
        debug!("Create post request received");

        // Используем user_id из токена
        let author_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| Status::internal("Invalid user ID in token"))?;

        let dto = CreatePostDto {
            title: req.title,
            content: req.data,
            author_id,
        };

        match self.post_app.create_post(dto).await {
            Ok(post_dto) => {
                info!("Post created successfully with id: {}", post_dto.uuid);
                Ok(Response::new(PostResponse {
                    response: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("Post created successfully".to_string()),
                    }),
                    post: Some(ProtoPost {
                        id: post_dto.uuid.to_string(),
                        title: post_dto.title,
                        data: post_dto.content,
                        created_ts: Some(Timestamp {
                            seconds: post_dto.created_at.timestamp(),
                            nanos: post_dto.created_at.timestamp_subsec_nanos() as i32,
                        }),
                        last_updated_ts: Some(Timestamp {
                            seconds: post_dto.updated_at.timestamp(),
                            nanos: post_dto.updated_at.timestamp_subsec_nanos() as i32,
                        }),
                    }),
                }))
            }
            Err(e) => {
                error!("Failed to create post: {}", e);
                Ok(Response::new(PostResponse {
                    response: Some(Self::map_domain_error(e)),
                    post: None,
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        // GetPost - публичный метод, не требует аутентификации
        let req = request.into_inner();
        debug!("Get post request received for id: {}", req.id);

        // Конвертируем строку в UUID
        let uuid = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;

        match self.post_app.get_post_by_id(uuid).await {
            Ok(post_dto) => {
                info!("Post retrieved successfully");
                Ok(Response::new(PostResponse {
                    response: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("Post retrieved successfully".to_string()),
                    }),
                    post: Some(ProtoPost {
                        id: post_dto.uuid.to_string(),
                        title: post_dto.title,
                        data: post_dto.content,
                        created_ts: Some(Timestamp {
                            seconds: post_dto.created_at.timestamp(),
                            nanos: post_dto.created_at.timestamp_subsec_nanos() as i32,
                        }),
                        last_updated_ts: Some(Timestamp {
                            seconds: post_dto.updated_at.timestamp(),
                            nanos: post_dto.updated_at.timestamp_subsec_nanos() as i32,
                        }),
                    }),
                }))
            }
            Err(e) => {
                warn!("Failed to retrieve post: {}", e);
                Ok(Response::new(PostResponse {
                    response: Some(Self::map_domain_error(e)),
                    post: None,
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        // Проверяем JWT токен
        let claims = self.auth_interceptor.verify_token(&request)?;
        debug!("Authenticated user: {}", claims.user_name);

        let req = request.into_inner();

        let post = req
            .post
            .ok_or_else(|| Status::invalid_argument("Post data is required"))?;

        debug!("Update post request received for id: {}", post.id);

        let uuid = Uuid::parse_str(&post.id)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;

        let dto = UpdatePostDto {
            uuid,
            title: post.title,
            content: post.data,
        };

        match self.post_app.update_post(dto).await {
            Ok(post_dto) => {
                info!("Post updated successfully");
                Ok(Response::new(PostResponse {
                    response: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("Post updated successfully".to_string()),
                    }),
                    post: Some(ProtoPost {
                        id: post_dto.uuid.to_string(),
                        title: post_dto.title,
                        data: post_dto.content,
                        created_ts: Some(Timestamp {
                            seconds: post_dto.created_at.timestamp(),
                            nanos: post_dto.created_at.timestamp_subsec_nanos() as i32,
                        }),
                        last_updated_ts: Some(Timestamp {
                            seconds: post_dto.updated_at.timestamp(),
                            nanos: post_dto.updated_at.timestamp_subsec_nanos() as i32,
                        }),
                    }),
                }))
            }
            Err(e) => {
                error!("Failed to update post: {}", e);
                Ok(Response::new(PostResponse {
                    response: Some(Self::map_domain_error(e)),
                    post: None,
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        // Проверяем JWT токен
        let claims = self.auth_interceptor.verify_token(&request)?;
        debug!("Authenticated user: {}", claims.user_name);

        let req = request.into_inner();
        debug!("Delete post request received for id: {}", req.post_id);

        let uuid = Uuid::parse_str(&req.post_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;

        match self.post_app.delete_post(uuid).await {
            Ok(_) => {
                info!("Post deleted successfully");
                Ok(Response::new(DeletePostResponse {
                    status: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("Post deleted successfully".to_string()),
                    }),
                }))
            }
            Err(e) => {
                error!("Failed to delete post: {}", e);
                Ok(Response::new(DeletePostResponse {
                    status: Some(Self::map_domain_error(e)),
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        // ListPosts - публичный метод, не требует аутентификации
        let _req = request.into_inner();
        debug!("List posts request received");

        match self.post_app.get_all_posts().await {
            Ok(posts) => {
                info!("Retrieved {} posts", posts.len());
                let proto_posts = posts
                    .into_iter()
                    .map(|post_dto| ProtoPost {
                        id: post_dto.uuid.to_string(),
                        title: post_dto.title,
                        data: post_dto.content,
                        created_ts: Some(Timestamp {
                            seconds: post_dto.created_at.timestamp(),
                            nanos: post_dto.created_at.timestamp_subsec_nanos() as i32,
                        }),
                        last_updated_ts: Some(Timestamp {
                            seconds: post_dto.updated_at.timestamp(),
                            nanos: post_dto.updated_at.timestamp_subsec_nanos() as i32,
                        }),
                    })
                    .collect();

                Ok(Response::new(ListPostsResponse {
                    status: Some(ProtoResponse {
                        code: ProtoStatus::Ok as i32,
                        details: Some("Posts retrieved successfully".to_string()),
                    }),
                    posts: proto_posts,
                }))
            }
            Err(e) => {
                error!("Failed to retrieve posts: {}", e);
                Ok(Response::new(ListPostsResponse {
                    status: Some(Self::map_domain_error(e)),
                    posts: vec![],
                }))
            }
        }
    }
}
