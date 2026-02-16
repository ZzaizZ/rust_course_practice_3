use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use uuid::Uuid;

use crate::{
    blog_client::BlogClient, error::ClientError, interceptor::decode_token_without_validation,
    token_manager::TokenManager, types,
};

pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
    token_manager: TokenManager,
}

impl HttpClient {
    pub async fn new(url: String) -> Result<Self, ClientError> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| ClientError::TransportError(e.to_string()))?;

        let base_url = url.trim_end_matches('/').to_string();

        Ok(Self {
            client,
            base_url,
            token_manager: TokenManager::new(300), // Обновлять токен за 5 минут до истечения
        })
    }

    pub async fn set_token(&self, token: String) {
        self.token_manager
            .set_auth_data(types::AuthData {
                access_token: token.clone(),
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
        let base_url = self.base_url.clone();
        self.token_manager
            .ensure_valid_token(|refresh_token| async move {
                Self::refresh_auth_token_internal(client, base_url, refresh_token).await
            })
            .await
    }

    /// Внутренний метод для обновления токена через HTTP
    async fn refresh_auth_token_internal(
        client: reqwest::Client,
        base_url: String,
        refresh_token: types::Token,
    ) -> types::ClientResult<types::AuthData> {
        let url = format!("{}/auth/refresh", base_url);

        let request_body = api::rest::RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
        };

        let response = client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ClientError::TransportError(format!(
                "Failed to refresh token: {}",
                response.status()
            )));
        }

        let token_response: api::rest::TokenResponse = response.json().await?;

        let access_token = token_response.access_token.clone();
        let new_refresh_token = token_response.refresh_token.clone();

        Ok(types::AuthData {
            access_token,
            refresh_token: new_refresh_token,
        })
    }

    /// Создает заголовки с токеном авторизации
    async fn create_headers(&self) -> Result<HeaderMap, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let auth_data = self.token_manager.get_auth_data().await;
        if let Some(data) = auth_data.as_ref() {
            let auth_value = format!("Bearer {}", data.access_token);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value).map_err(|e| {
                    ClientError::InternalError(format!("Invalid header value: {}", e))
                })?,
            );
        }

        Ok(headers)
    }

    /// Обрабатывает ошибку HTTP-ответа
    async fn handle_error_response(response: reqwest::Response) -> ClientError {
        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return ClientError::Unauthorized;
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return ClientError::NotFound;
        }

        if status.is_client_error() {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return ClientError::InvalidRequest(error_msg);
        }

        let error_msg = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        ClientError::InternalError(error_msg)
    }
}

#[async_trait]
impl BlogClient for HttpClient {
    async fn login(&self, username: &str, password: &str) -> types::ClientResult<Uuid> {
        let url = format!("{}/api/v1/auth/login", self.base_url);

        let request_body = api::rest::LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let token_response: api::rest::TokenResponse = response.json().await?;

        // Создаем и сохраняем токены
        let access_token = token_response.access_token.clone();
        let refresh_token = token_response.refresh_token.clone();

        let auth_data = types::AuthData {
            access_token: access_token.clone(),
            refresh_token,
        };

        // Сохраняем токены в auth_data
        self.token_manager.set_auth_data(auth_data).await;

        // Декодируем токен для получения user ID
        let user_id = decode_token_without_validation(&access_token)
            .ok()
            .and_then(|claims| Uuid::parse_str(&claims.sub).ok())
            .unwrap_or(Uuid::nil());

        Ok(user_id)
    }

    async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> types::ClientResult<()> {
        let url = format!("{}/api/v1/auth/register", self.base_url);

        let request_body = api::rest::RegisterRequest {
            username: username.to_string(),
            password: password.to_string(),
            email: email.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        Ok(())
    }

    async fn setup_token(&self, token: &str) -> types::ClientResult<()> {
        self.set_token(token.to_string()).await;
        self.ensure_valid_token().await
    }

    async fn get_token(&self) -> types::ClientResult<Option<String>> {
        Ok(self.token_manager.get_access_token().await)
    }

    async fn create_post(&self, title: &str, content: &str) -> types::ClientResult<Uuid> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let url = format!("{}/api/v1/posts", self.base_url);
        let headers = self.create_headers().await?;

        let request_body = api::rest::CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let post_response: api::rest::PostResponse = response.json().await?;

        let id = Uuid::parse_str(&post_response.uuid)
            .map_err(|e| ClientError::InternalError(format!("Invalid UUID: {}", e)))?;

        Ok(id)
    }

    async fn get_post(&self, post_id: &str) -> types::ClientResult<types::Post> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let url = format!("{}/api/v1/posts/{}", self.base_url, post_id);
        let headers = self.create_headers().await?;

        let response = self.client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let post_response: api::rest::PostResponse = response.json().await?;

        let id = Uuid::parse_str(&post_response.uuid)
            .map_err(|e| ClientError::InternalError(format!("Invalid UUID: {}", e)))?;

        let created_at = DateTime::parse_from_rfc3339(&post_response.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&post_response.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(types::Post {
            id,
            title: post_response.title,
            content: post_response.content,
            created_at,
            updated_at,
        })
    }

    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> types::ClientResult<()> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let url = format!("{}/api/v1/posts/{}", self.base_url, post_id);
        let headers = self.create_headers().await?;

        let request_body = api::rest::UpdatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        };

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        Ok(())
    }

    async fn delete_post(&self, post_id: &str) -> types::ClientResult<()> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let url = format!("{}/api/v1/posts/{}", self.base_url, post_id);
        let headers = self.create_headers().await?;

        let response = self.client.delete(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        Ok(())
    }

    async fn list_posts(&self, page_size: u8, page: u32) -> types::ClientResult<Vec<types::Post>> {
        // Проверяем и обновляем токен при необходимости
        self.ensure_valid_token().await?;

        let url = format!(
            "{}/api/v1/posts?page_size={}&page={}",
            self.base_url, page_size, page
        );
        let headers = self.create_headers().await?;

        let response = self.client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let posts_response: Vec<api::rest::PostResponse> = response.json().await?;

        posts_response
            .into_iter()
            .map(|post_response| {
                let id = Uuid::parse_str(&post_response.uuid)
                    .map_err(|e| ClientError::InternalError(format!("Invalid UUID: {}", e)))?;

                let created_at = DateTime::parse_from_rfc3339(&post_response.created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let updated_at = DateTime::parse_from_rfc3339(&post_response.updated_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(types::Post {
                    id,
                    title: post_response.title,
                    content: post_response.content,
                    created_at,
                    updated_at,
                })
            })
            .collect()
    }
}
