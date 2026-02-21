use serde::Deserialize;
use serde_yml;

/// Конфигурация сервера.
///
/// Содержит все настройки, необходимые для запуска и работы сервера.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Строка подключения к PostgreSQL БД
    pub db_connection_string: String,
    /// Секретный ключ для подписи JWT токенов
    pub jwt_secret: String,
    /// Время жизни JWT access токена в секундах
    pub jwt_expiration_seconds: i64,
    /// Порт HTTP сервера
    pub server_port: u16,
    /// Порт gRPC сервера
    pub grpc_port: u16,
    /// Разрешённый CORS origin
    pub cors_origin: String,
    /// Уровень логирования (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Config {
    /// Загружает конфигурацию из YAML файла.
    ///
    /// # Аргументы
    ///
    /// * `path` - Путь к файлу конфигурации
    ///
    /// # Ошибки
    ///
    /// Возвращает ошибку если файл не найден или содержит невалидный YAML
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yml::from_str(&config_str)?;
        Ok(config)
    }

    /// Загружает конфигурацию из переменных окружения.
    ///
    /// # Переменные окружения
    ///
    /// - `DB_CONNECTION_STRING` - строка подключения к БД (обязательна)
    /// - `JWT_SECRET` - секрет для JWT (обязательна)
    /// - `JWT_EXPIRATION_SECONDS` - время жизни токена (обязательна)
    /// - `SERVER_PORT` - порт HTTP сервера (обязательна)
    /// - `GRPC_PORT` - порт gRPC сервера (по умолчанию: 50051)
    /// - `CORS_ORIGIN` - разрешённый origin (обязательна)
    /// - `LOG_LEVEL` - уровень логов (по умолчанию: info)
    ///
    /// # Ошибки
    ///
    /// Паникует если обязательные переменные не установлены
    pub fn from_env() -> anyhow::Result<Self> {
        let db_connection_string =
            std::env::var("DB_CONNECTION_STRING").expect("DB_CONNECTION_STRING must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expiration_seconds = std::env::var("JWT_EXPIRATION_SECONDS")
            .expect("JWT_EXPIRATION_SECONDS must be set")
            .parse::<i64>()?;
        let server_port = std::env::var("SERVER_PORT")
            .expect("SERVER_PORT must be set")
            .parse::<u16>()?;
        let grpc_port = std::env::var("GRPC_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse::<u16>()?;
        let cors_origin = std::env::var("CORS_ORIGIN").expect("CORS_ORIGIN must be set");
        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            db_connection_string,
            jwt_secret,
            jwt_expiration_seconds,
            server_port,
            grpc_port,
            cors_origin,
            log_level,
        })
    }
}
