use crate::types;
use async_trait::async_trait;
use uuid::Uuid;

/// Тип транспорта для клиента.
pub enum Transport {
    /// gRPC транспорт с указанием URL сервера
    Grpc(String),
    /// HTTP транспорт с указанием базового URL
    Http(String),
}

// Для не-WASM требуется Send для поддержки многопоточности
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Основной трейт клиента для взаимодействия с блог-платформой.
///
/// Определяет единый интерфейс для всех типов клиентов (HTTP, gRPC).
/// В нативной среде требует `Send` для поддержки многопоточности.
///
/// # Методы аутентификации
///
/// * [`login`](BlogClient::login) - Вход пользователя
/// * [`register`](BlogClient::register) - Регистрация нового пользователя
/// * [`setup_token`](BlogClient::setup_token) - Установка токена вручную
/// * [`get_token`](BlogClient::get_token) - Получение текущего токена
///
/// # CRUD операции с постами
///
/// * [`create_post`](BlogClient::create_post) - Создание нового поста
/// * [`get_post`](BlogClient::get_post) - Получение поста по ID
/// * [`update_post`](BlogClient::update_post) - Обновление поста
/// * [`delete_post`](BlogClient::delete_post) - Удаление поста
/// * [`list_posts`](BlogClient::list_posts) - Получение списка постов с пагинацией
pub trait BlogClient {
    /// Выполняет вход пользователя в систему.
    async fn login(&self, username: &str, password: &str) -> types::ClientResult<Uuid>;
    /// Регистрирует нового пользователя.
    async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> types::ClientResult<()>;
    /// Устанавливает JWT токен для аутентификации запросов.
    async fn setup_token(&self, token: &str) -> types::ClientResult<()>;
    /// Возвращает текущий JWT токен, если он установлен.
    async fn get_token(&self) -> types::ClientResult<Option<String>>;
    /// Устанавливает полные данные аутентификации (access и refresh токены).
    async fn setup_auth_data(&self, auth_data: &types::AuthData) -> types::ClientResult<()>;
    /// Возвращает полные данные аутентификации, если они установлены.
    async fn get_auth_data(&self) -> types::ClientResult<Option<types::AuthData>>;

    /// Создаёт новый пост в блоге (требуется аутентификация).
    async fn create_post(&self, title: &str, content: &str) -> types::ClientResult<Uuid>;
    /// Получает пост по его ID.
    async fn get_post(&self, post_id: &str) -> types::ClientResult<types::Post>;
    /// Обновляет существующий пост (требуется быть автором).
    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> types::ClientResult<()>;
    /// Удаляет пост (требуется быть автором).
    async fn delete_post(&self, post_id: &str) -> types::ClientResult<()>;
    /// Получает список постов с пагинацией.
    async fn list_posts(&self, page_size: u32, page: u32) -> types::ClientResult<Vec<types::Post>>;
}
