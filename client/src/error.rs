use thiserror::Error;

/// Ошибки клиента при взаимодействии с API.
#[derive(Error, Debug)]
pub enum ClientError {
    /// Ресурс не найден (HTTP 404)
    #[error("Not Found")]
    NotFound,

    /// Ошибка аутентификации (HTTP 401)
    #[error("Unauthorized")]
    Unauthorized,

    /// Невалидный запрос (HTTP 400)
    #[error("Invalid Request: {0}")]
    InvalidRequest(String),

    /// Внутренняя ошибка сервера (HTTP 500)
    #[error("Internal Error: {0}")]
    InternalError(String),

    /// Ошибка на уровне транспорта (сеть, протокол)
    #[error("Transport Error: {0}")]
    TransportError(String),
}

#[cfg(feature = "grpc")]
impl From<tonic::transport::Error> for ClientError {
    fn from(value: tonic::transport::Error) -> Self {
        ClientError::TransportError(value.to_string())
    }
}

#[cfg(feature = "grpc")]
impl From<tonic::Status> for ClientError {
    fn from(value: tonic::Status) -> Self {
        ClientError::TransportError(value.to_string())
    }
}

#[cfg(feature = "http")]
impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::TransportError(value.to_string())
    }
}
