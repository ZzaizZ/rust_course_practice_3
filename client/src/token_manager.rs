use std::sync::Arc;

use tokio::sync::{Mutex, RwLock, mpsc};

use crate::{error::ClientError, interceptor::decode_token_without_validation, types};

/// Событие об обновлении токена
#[derive(Debug, Clone)]
pub struct TokenUpdateEvent {
    pub access_token: String,
}

/// Менеджер токенов с автоматической проверкой и обновлением
#[derive(Clone)]
pub struct TokenManager {
    auth_data: Arc<RwLock<Option<types::AuthData>>>,
    token_refresh_buffer_seconds: i64,
    refresh_lock: Arc<Mutex<()>>,
    token_update_sender: Option<mpsc::UnboundedSender<TokenUpdateEvent>>,
}

impl TokenManager {
    pub fn new(token_refresh_buffer_seconds: i64) -> Self {
        Self {
            auth_data: Arc::new(RwLock::new(None)),
            token_refresh_buffer_seconds,
            refresh_lock: Arc::new(Mutex::new(())),
            token_update_sender: None,
        }
    }

    /// Создает TokenManager с channel для уведомлений об обновлении токена
    pub fn new_with_notifier(
        token_refresh_buffer_seconds: i64,
        sender: mpsc::UnboundedSender<TokenUpdateEvent>,
    ) -> Self {
        Self {
            auth_data: Arc::new(RwLock::new(None)),
            token_refresh_buffer_seconds,
            refresh_lock: Arc::new(Mutex::new(())),
            token_update_sender: Some(sender),
        }
    }

    /// Устанавливает данные аутентификации
    pub async fn set_auth_data(&self, auth_data: types::AuthData) {
        let access_token = auth_data.access_token.clone();
        let mut data = self.auth_data.write().await;
        *data = Some(auth_data);

        // Уведомляем об обновлении токена
        if let Some(sender) = &self.token_update_sender {
            let _ = sender.send(TokenUpdateEvent { access_token });
        }
    }

    /// Получает access token
    pub async fn get_access_token(&self) -> Option<String> {
        let auth_data = self.auth_data.read().await;
        auth_data.as_ref().map(|data| data.access_token.clone())
    }

    /// Получает refresh token
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

                            let access_token = new_auth_data.access_token.clone();
                            let mut auth_data_write = self.auth_data.write().await;
                            *auth_data_write = Some(new_auth_data);

                            // Уведомляем об обновлении токена
                            if let Some(sender) = &self.token_update_sender {
                                let _ = sender.send(TokenUpdateEvent { access_token });
                            }
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
