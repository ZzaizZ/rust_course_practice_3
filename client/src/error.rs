use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Not Found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid Request: {0}")]
    InvalidRequest(String),
    #[error("Internal Error: {0}")]
    InternalError(String),
    #[error("Transport Error: {0}")]
    TransportError(String),
}

impl From<tonic::transport::Error> for ClientError {
    fn from(value: tonic::transport::Error) -> Self {
        ClientError::TransportError(value.to_string())
    }
}

impl From<tonic::Status> for ClientError {
    fn from(value: tonic::Status) -> Self {
        ClientError::TransportError(value.to_string())
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::TransportError(value.to_string())
    }
}
