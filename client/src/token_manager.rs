use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::{error::ClientError, interceptor::decode_token_without_validation, types};

/// Менеджер токенов с автоматической проверкой и обновлением
pub struct TokenManager {
    auth_data: Arc<RwLock<Option<types::AuthData>>>,
    token_refresh_buffer_seconds: i64,
    refresh_lock: Arc<Mutex<()>>,
}

impl TokenManager {
    pub fn new(token_refresh_buffer_seconds: i64) -> Self {
        Self {
            auth_data: Arc::new(RwLock::new(None)),
            token_refresh_buffer_seconds,
            refresh_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Устанавливает данные аутентификации
    pub async fn set_auth_data(&self, auth_data: types::AuthData) {
        let mut data = self.auth_data.write().await;
        *data = Some(auth_data);
    }

    /// Получает access token
    pub async fn get_access_token(&self) -> Option<String> {
        let auth_data = self.auth_data.read().await;
        auth_data.as_ref().map(|data| data.access_token.clone())
    }

    /// Получает refresh token
    #[allow(dead_code)]
    pub async fn get_refresh_token(&self) -> Option<String> {
        let auth_data = self.auth_data.read().await;
        auth_data.as_ref().map(|data| data.refresh_token.clone())
    }

    /// Получает полные данные аутентификации
    pub async fn get_auth_data(&self) -> Option<types::AuthData> {
        let auth_data = self.auth_data.read().await;
        auth_data.clone()
    }

    /// Устанавливает буфер времени для обновления токена (в секундах)
    pub fn set_token_refresh_buffer(&mut self, seconds: i64) {
        self.token_refresh_buffer_seconds = seconds;
    }

    /// Проверяет токен и обновляет его при необходимости
    /// Использует мьютекс для предотвращения одновременного обновления токена несколькими запросами
    pub async fn ensure_valid_token<F, Fut>(&self, refresh_fn: F) -> Result<(), ClientError>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<types::AuthData, ClientError>>,
    {
        // Сначала быстро проверяем без блокировки
        let auth_data_clone = self.auth_data.read().await.clone();

        if let Some(data) = auth_data_clone {
            // Декодируем access token для проверки времени жизни
            match decode_token_without_validation(&data.access_token) {
                Ok(claims) => {
                    // Если токен истекает скоро, обновляем его
                    if claims.expires_soon(self.token_refresh_buffer_seconds) {
                        // Захватываем мьютекс, чтобы только один поток мог обновлять токен
                        let _guard = self.refresh_lock.lock().await;

                        // Проверяем токен еще раз после захвата мьютекса
                        // (возможно, другой поток уже обновил его)
                        let current_auth_data = self.auth_data.read().await.clone();
                        if let Some(current_data) = current_auth_data {
                            if let Ok(current_claims) =
                                decode_token_without_validation(&current_data.access_token)
                            {
                                // Если токен уже обновлен другим потоком, не обновляем снова
                                if !current_claims.expires_soon(self.token_refresh_buffer_seconds) {
                                    return Ok(());
                                }
                            }

                            // Обновляем токен через переданную функцию
                            let new_auth_data = refresh_fn(current_data.refresh_token).await?;

                            let mut auth_data_write = self.auth_data.write().await;
                            *auth_data_write = Some(new_auth_data);
                        }
                    }
                }
                Err(e) => {
                    // Если не можем декодировать токен, возвращаем ошибку
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_manager_set_get() {
        let manager = TokenManager::new(300);

        let auth_data = types::AuthData {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
        };

        manager.set_auth_data(auth_data.clone()).await;

        let token = manager.get_access_token().await;
        assert_eq!(token, Some("test_access".to_string()));

        let refresh = manager.get_refresh_token().await;
        assert_eq!(refresh, Some("test_refresh".to_string()));
    }
}
