use api::blog_client::BlogClient as BlogGrpcClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use tonic::Request;
use tonic::metadata::MetadataValue;
use uuid::Uuid;

use crate::{
    blog_client::BlogClient, error::ClientError, interceptor::decode_token_without_validation,
    token_manager::TokenManager, types,
};

pub struct GrpcClient {
    client: BlogGrpcClient<tonic::transport::Channel>,
    token_manager: TokenManager,
}

impl GrpcClient {
    pub async fn new(url: String) -> Result<Self, ClientError> {
        let client = BlogGrpcClient::connect(url).await?;
        Ok(Self {
            client,
            token_manager: TokenManager::new(300), // Обновлять токен за 5 минут до истечения
        })
    }

    pub async fn set_token(&self, token: &str) {
        self.token_manager
            .set_auth_data(types::AuthData {
                access_token: token.to_string(),
                refresh_token: String::new(),
            })
            .await;
    }

    pub async fn get_token(&self) -> Option<String> {
        self.token_manager.get_access_token().await
    }

    /// Устанавливает буфер времени для обновления токена (в секундах)
    pub fn set_token_refresh_buffer(&mut self, seconds: i64) {
        self.token_manager.set_token_refresh_buffer(seconds);
    }

    /// Проверяет токен и обновляет его при необходимости
    async fn ensure_valid_token(&self) -> Result<(), ClientError> {
        let client = self.client.clone();
        self.token_manager
            .ensure_valid_token(|refresh_token| async move {
                Self::refresh_auth_token_internal(client, refresh_token).await
            })
            .await
    }

    /// Внутренний метод для обновления токена через gRPC
    async fn refresh_auth_token_internal(
        mut client: BlogGrpcClient<tonic::transport::Channel>,
        refresh_token: types::Token,
    ) -> types::ClientResult<types::AuthData> {
        let request = Request::new(api::RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
        });

        let response = client.refresh_token(request).await?.into_inner();

        check_response(response.status.clone())?;

        let token_container = response
            .token
            .ok_or_else(|| ClientError::InternalError("No token in response".to_string()))?;

        Ok(types::AuthData {
            access_token: token_container.access_token,
            refresh_token: token_container.refresh_token,
        })
    }

    async fn create_request<T>(&self, message: T) -> Result<Request<T>, ClientError> {
        let auth_data = self.token_manager.get_auth_data().await;
        let mut request = Request::new(message);

        if let Some(data) = auth_data.as_ref()
            && let Ok(token_value) =
                MetadataValue::try_from(format!("Bearer {}", data.access_token))
        {
            request.metadata_mut().insert("authorization", token_value);
        }

        Ok(request)
    }

    fn create_request_without_token<T>(&self, message: T) -> Request<T> {
        Request::new(message)
    }
}

// Helper functions to convert between protobuf and chrono timestamps
fn timestamp_to_datetime(ts: Option<Timestamp>) -> DateTime<Utc> {
    ts.and_then(|t| DateTime::from_timestamp(t.seconds, t.nanos as u32))
        .unwrap_or_else(Utc::now)
}

fn datetime_to_timestamp(dt: DateTime<Utc>) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

fn proto_post_to_client_post(post: api::Post) -> Result<types::Post, ClientError> {
    let id = Uuid::parse_str(&post.id)
        .map_err(|e| ClientError::InternalError(format!("Invalid UUID: {}", e)))?;

    Ok(types::Post {
        id,
        title: post.title,
        content: post.data,
        created_at: timestamp_to_datetime(post.created_ts),
        updated_at: timestamp_to_datetime(post.last_updated_ts),
    })
}

fn check_response(response: Option<api::Response>) -> Result<(), ClientError> {
    let response = response.ok_or_else(|| ClientError::InternalError("No response".to_string()))?;

    match response.code() {
        api::Status::Ok => Ok(()),
        api::Status::Unauthorized => Err(ClientError::Unauthorized),
        api::Status::InvalidRequest => Err(ClientError::InvalidRequest(
            response.details.unwrap_or_default(),
        )),
        api::Status::InternalError => Err(ClientError::InternalError(
            response.details.unwrap_or_default(),
        )),
    }
}

#[async_trait]
impl BlogClient for GrpcClient {
    async fn login(&self, username: &str, password: &str) -> types::ClientResult<Uuid> {
        let request = self.create_request_without_token(api::LoginRequest {
            email_or_login: username.to_string(),
            password: password.to_string(),
        });

        let response = self.client.clone().login(request).await?.into_inner();

        check_response(response.status)?;

        let token_container = response
            .token
            .ok_or_else(|| ClientError::InternalError("No token in response".to_string()))?;

        // Декодируем токен для получения user ID
        let user_id = decode_token_without_validation(&token_container.access_token)
            .ok()
            .and_then(|claims| Uuid::parse_str(&claims.sub).ok())
            .unwrap_or(Uuid::nil());

        // Сохраняем токены для последующих запросов
        let access_token = token_container.access_token.clone();
        let refresh_token = token_container.refresh_token.clone();

        let auth_data = types::AuthData {
            access_token: access_token.clone(),
            refresh_token,
        };

        self.token_manager.set_auth_data(auth_data).await;

        Ok(user_id)
    }

    async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> types::ClientResult<()> {
        let request = self.create_request_without_token(api::RegisterRequest {
            login: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.client.clone().register(request).await?.into_inner();

        check_response(response.status)
    }

    async fn setup_token(&self, token: &str) -> types::ClientResult<()> {
        self.set_token(token).await;
        self.ensure_valid_token().await
    }

    async fn get_token(&self) -> types::ClientResult<Option<String>> {
        Ok(self.get_token().await)
    }

    async fn create_post(&self, title: &str, content: &str) -> types::ClientResult<Uuid> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let request = self
            .create_request(api::CreatePostRequest {
                title: title.to_string(),
                data: content.to_string(),
            })
            .await?;

        let response = self.client.clone().create_post(request).await?.into_inner();

        check_response(response.response.clone())?;

        let post = response
            .post
            .ok_or_else(|| ClientError::InternalError("No post in response".to_string()))?;

        let id = Uuid::parse_str(&post.id)
            .map_err(|e| ClientError::InternalError(format!("Invalid UUID: {}", e)))?;

        Ok(id)
    }

    async fn get_post(&self, post_id: &str) -> types::ClientResult<types::Post> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let request = self
            .create_request(api::GetPostRequest {
                id: post_id.to_string(),
            })
            .await?;

        let response = self.client.clone().get_post(request).await?.into_inner();

        check_response(response.response)?;

        let post = response.post.ok_or(ClientError::NotFound)?;

        proto_post_to_client_post(post)
    }

    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> types::ClientResult<()> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let request = self
            .create_request(api::UpdatePostRequest {
                post: Some(api::Post {
                    id: post_id.to_string(),
                    title: title.to_string(),
                    data: content.to_string(),
                    created_ts: None,
                    last_updated_ts: datetime_to_timestamp(Utc::now()),
                }),
            })
            .await?;

        let response = self.client.clone().update_post(request).await?.into_inner();

        check_response(response.response)
    }

    async fn delete_post(&self, post_id: &str) -> types::ClientResult<()> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let request = self
            .create_request(api::DeletePostRequest {
                post_id: post_id.to_string(),
            })
            .await?;

        let response = self.client.clone().delete_post(request).await?.into_inner();

        check_response(response.status)
    }

    async fn list_posts(&self, page_size: u8, page: u32) -> types::ClientResult<Vec<types::Post>> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let request = self
            .create_request(api::ListPostsRequest {
                page_count: page as i32,
                page_size: page_size as i32,
            })
            .await?;

        let response = self.client.clone().list_posts(request).await?.into_inner();

        check_response(response.status)?;

        response
            .posts
            .into_iter()
            .map(proto_post_to_client_post)
            .collect()
    }
}
