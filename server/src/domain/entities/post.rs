use uuid::Uuid;

/// Доменная сущность поста блога.
///
/// Представляет публикацию в блоге с заголовком, содержимым и метаданными.
///
/// # Поля
///
/// * `uuid` - Уникальный идентификатор поста
/// * `title` - Заголовок поста
/// * `content` - Содержимое поста
/// * `author_id` - ID автора (ссылка на User)
/// * `created_at` - Временная метка создания
/// * `updated_at` - Временная метка последнего обновления
#[derive(Debug, Clone)]
pub struct Post {
    pub uuid: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
