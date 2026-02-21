use client::types::AuthData;
use gloo_storage::{LocalStorage, Storage};

const TOKEN_STORAGE_KEY: &str = "auth_token";

/// Сохраняет данные аутентификации (access и refresh токены) в localStorage
pub fn save_auth_data(auth_data: &AuthData) -> Result<(), String> {
    LocalStorage::set(TOKEN_STORAGE_KEY, auth_data)
        .map_err(|e| format!("Failed to save auth data: {:?}", e))
}

/// Загружает данные аутентификации из localStorage
pub fn load_auth_data() -> Option<AuthData> {
    LocalStorage::get::<AuthData>(TOKEN_STORAGE_KEY).ok()
}

/// Удаляет данные аутентификации из localStorage
pub fn clear_auth_data() {
    LocalStorage::delete(TOKEN_STORAGE_KEY);
}
