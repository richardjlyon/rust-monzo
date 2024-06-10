use thiserror::Error;

use crate::client::ErrorJson;

// use crate::client::MonzoClientError;

#[derive(Debug, Error)]
pub enum AppError {
    // -- Authorisation
    #[error("Access token error")]
    AccessTokenError(String),

    #[error("Failed to exchange auth code for access token")]
    AuthCodeExchangeError,

    #[error("Authorisation failure: {0}")]
    AuthorisationFailure(#[from] ErrorJson),

    // -- Server error
    #[error("Handler error")]
    HandlerError,

    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Server error")]
    ServerError,

    // -- File error
    #[error("Failed to open file")]
    FileError(#[from] std::io::Error),

    #[error("Failed to deserialise yaml")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Configuration error")]
    ConfigurationError(#[from] config::ConfigError),

    // -- Database error
    #[error("Query error")]
    QueryError(#[from] sqlx::Error),

    #[error("Query error {0}")]
    Duplicate(String),

    #[error("Database error")]
    DbError(String),

    // -- Command error
    #[error("Command aborted")]
    AbortError,
}
