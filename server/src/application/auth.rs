use crate::application::dto::auth::{LoginDto, RegisterDto, TokenDto};
use crate::domain::entities::{errors::DomainResult, user::User};
use crate::domain::repositories::repo::UserRepository;
use crate::domain::services::auth::AuthService;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

pub struct AuthApplication<Repo: UserRepository> {
    user_repository: Arc<Repo>,
    auth_service: Arc<AuthService>,
}

impl<Repo: UserRepository> AuthApplication<Repo> {
    pub fn new(user_repository: Arc<Repo>, auth_service: Arc<AuthService>) -> Self {
        Self {
            user_repository,
            auth_service,
        }
    }

    #[instrument(skip(self, dto), fields(username = %dto.username, email = %dto.email))]
    pub async fn create_user(&self, dto: RegisterDto) -> DomainResult<User> {
        debug!("Attempting to create new user");

        if self
            .user_repository
            .exists_by_username(&dto.username)
            .await?
        {
            warn!("User registration failed: username already exists");
            return Err(
                crate::domain::entities::errors::DomainError::UserAlreadyExists {
                    username: dto.username,
                },
            );
        }

        let password_hash = self
            .auth_service
            .hash_password(&dto.password)
            .map_err(|e| {
                warn!("Password hashing failed: {}", e);
                crate::domain::entities::errors::DomainError::InvalidPassword {
                    reason: e.to_string(),
                }
            })?;

        let user = User::new(
            Uuid::now_v7(),
            dto.username.clone(),
            dto.email,
            password_hash,
            chrono::Utc::now(),
        );

        let created_user = self.user_repository.create_user(user).await?;
        info!("User created successfully with id: {}", created_user.id);

        Ok(created_user)
    }

    #[instrument(skip(self, dto), fields(username = %dto.username))]
    pub async fn login(&self, dto: LoginDto) -> DomainResult<TokenDto> {
        debug!("Attempting user login");

        // Найти пользователя
        let user = self
            .user_repository
            .find_by_username(&dto.username)
            .await?
            .ok_or_else(|| {
                warn!("Login failed: user not found");
                crate::domain::entities::errors::DomainError::UserNotFound {
                    username: dto.username.clone(),
                }
            })?;

        if !self
            .auth_service
            .verify_password(&dto.password, &user.password_hash)
        {
            warn!("Login failed: invalid credentials for user");
            return Err(crate::domain::entities::errors::DomainError::InvalidCredentials);
        }

        let access_token = self
            .auth_service
            .generate_token(&user.id.to_string(), &user.username);

        let refresh_token = self
            .auth_service
            .generate_refresh_token(&user.id.to_string(), &user.username);

        info!("User logged in successfully");

        Ok(TokenDto {
            access_token,
            refresh_token,
            expires_in: 86400,
        })
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn refresh_token(&self, refresh_token: String) -> DomainResult<TokenDto> {
        debug!("Attempting to refresh token");

        let claims = self
            .auth_service
            .verify_token(&refresh_token)
            .ok_or_else(|| {
                warn!("Token refresh failed: invalid refresh token");
                crate::domain::entities::errors::DomainError::TokenValidationError(
                    "Invalid refresh token".to_string(),
                )
            })?;

        let access_token = self
            .auth_service
            .generate_token(&claims.sub, &claims.user_name);

        // Генерируем новый refresh token
        let new_refresh_token = self
            .auth_service
            .generate_refresh_token(&claims.sub, &claims.user_name);

        info!(
            "Token refreshed successfully for user: {}",
            claims.user_name
        );

        Ok(TokenDto {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: 86400,
        })
    }
}
