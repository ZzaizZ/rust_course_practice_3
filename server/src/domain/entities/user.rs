use uuid::Uuid;

/// Доменная сущность пользователя системы.
///
/// # Поля
///
/// * `id` - Уникальный идентификатор пользователя (UUID v7)
/// * `username` - Уникальное имя пользователя для аутентификации и отображения
/// * `email` - Email пользователя (уникальный, используется для входа)
/// * `password_hash` - Хэш пароля (Argon2id)
/// * `created_at` - Временная метка создания пользователя
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    /// Создаёт новый экземпляр User.
    ///
    /// # Аргументы
    ///
    /// * `id` - Уникальный идентификатор (обычно UUID v7)
    /// * `username` - Имя пользователя (должно быть уникальным в системе)
    /// * `email` - Email адрес (должен быть уникальным в системе)
    /// * `password_hash` - Предварительно захэшированный пароль (с использованием Argon2id)
    /// * `created_at` - Временная метка создания
    pub fn new(
        id: Uuid,
        username: String,
        email: String,
        password_hash: String,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            created_at,
        }
    }
}
