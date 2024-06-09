use thiserror::Error;

use crate::client::ErrorJson;

// use crate::client::MonzoClientError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Server error")]
    ServerError,

    #[error("Access token error")]
    AccessTokenError(String),

    #[error("Failed to exchange auth code for access token")]
    AuthCodeExchangeError,

    #[error("Failed to open file")]
    FileError(#[from] std::io::Error),

    #[error("Failed to deserialise yaml")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Authorisation failure: {0}")]
    AuthorisationFailure(#[from] ErrorJson),

    #[error("Handler error")]
    HandlerError,

    #[error("Query error")]
    QueryError(#[from] sqlx::Error),

    #[error("Configuration error")]
    ConfigurationError(#[from] config::ConfigError),
}
