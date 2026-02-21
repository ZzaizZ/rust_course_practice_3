use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

/// Claims (полезная нагрузка) JWT токена.
///
/// Содержит информацию о пользователе и времени действия токена.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// ID пользователя (subject)
    pub sub: String,
    /// Имя пользователя
    pub user_name: String,
    /// Время истечения токена (Unix timestamp)
    pub exp: usize,
    /// Время выдачи токена (Unix timestamp)
    pub iat: u64,
}

/// Сервис аутентификации и авторизации.
///
/// Предоставляет функциональность для:
/// - Хэширования и проверки паролей (Argon2id)
/// - Генерации и валидации JWT токенов
/// - Управления refresh токенами
pub struct AuthService {
    password_hasher: Argon2<'static>,
    token_expiry_duration: chrono::Duration,
    secret: Vec<u8>,
}

impl AuthService {
    /// Создаёт новый экземпляр AuthService.
    ///
    /// # Аргументы
    ///
    /// * `token_expiry_duration` - Длительность жизни access токена
    /// * `secret` - Секретный ключ для подписи JWT токенов
    ///
    /// # Примечание
    ///
    /// Использует Argon2id с параметрами:
    /// - memory cost: 19 MiB
    /// - time cost: 2 iterations
    /// - parallelism: 1 thread
    pub fn new(token_expiry_duration: chrono::Duration, secret: &[u8]) -> Self {
        let params =
            Params::new(19 * 1024, 2, 1, None).expect("Failed to create Argon2 parameters");
        let password_hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        Self {
            password_hasher,
            token_expiry_duration,
            secret: secret.to_vec(),
        }
    }

    /// Хэширует пароль с использованием Argon2id.
    ///
    /// # Аргументы
    ///
    /// * `password` - Пароль в открытом виде
    ///
    /// # Возвращает
    ///
    /// Строку с хэшем пароля в формате PHC (includes salt and parameters)
    ///
    /// # Ошибки
    ///
    /// Возвращает ошибку если не удалось сгенерировать соль или захэшировать пароль
    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .password_hasher
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    /// Проверяет соответствие пароля хэшу.
    ///
    /// # Аргументы
    ///
    /// * `password` - Пароль в открытом виде
    /// * `password_hash` - Хэш пароля для проверки
    ///
    /// # Возвращает
    ///
    /// `true` если пароль совпадает, `false` в противном случае
    pub fn verify_password(&self, password: &str, password_hash: &str) -> bool {
        let parsed_hash = match argon2::PasswordHash::new(password_hash) {
            Ok(hash) => hash,
            Err(_) => return false,
        };
        self.password_hasher
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// Генерирует access токен для пользователя.
    ///
    /// # Аргументы
    ///
    /// * `user_id` - ID пользователя
    /// * `user_name` - Имя пользователя
    ///
    /// # Возвращает
    ///
    /// JWT токен в виде строки
    ///
    /// # Паника
    ///
    /// Паникует если не удалось создать токен (проблемы с кодированием)
    pub fn generate_token(&self, user_id: &str, user_name: &str) -> String {
        let now = chrono::offset::Utc::now();

        let claims = Claims {
            sub: user_id.to_string(),
            user_name: user_name.to_string(),
            exp: (now + self.token_expiry_duration).timestamp() as usize,
            iat: now.timestamp() as u64,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .expect("Failed to encode token")
    }

    /// Генерирует refresh токен для пользователя.
    ///
    /// Refresh токен живёт 30 дней и используется для получения нового access токена.
    ///
    /// # Аргументы
    ///
    /// * `user_id` - ID пользователя
    /// * `user_name` - Имя пользователя
    ///
    /// # Возвращает
    ///
    /// JWT refresh токен в виде строки
    pub fn generate_refresh_token(&self, user_id: &str, user_name: &str) -> String {
        let now = chrono::offset::Utc::now();
        // Refresh token живет 30 дней
        let refresh_expiry = chrono::Duration::days(30);

        let claims = Claims {
            sub: user_id.to_string(),
            user_name: user_name.to_string(),
            exp: (now + refresh_expiry).timestamp() as usize,
            iat: now.timestamp() as u64,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .expect("Failed to encode refresh token")
    }

    /// Проверяет и декодирует JWT токен.
    ///
    /// # Аргументы
    ///
    /// * `token` - JWT токен для проверки
    ///
    /// # Возвращает
    ///
    /// `Some(Claims)` если токен валиден, `None` если токен невалиден или истёк
    pub fn verify_token(&self, token: &str) -> Option<Claims> {
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(self.secret.as_ref());
        let validation = jsonwebtoken::Validation::default();
        match jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => Some(token_data.claims),
            Err(_) => None,
        }
    }
}
