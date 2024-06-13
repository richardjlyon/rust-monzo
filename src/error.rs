use thiserror::Error;

use crate::client::ErrorJson;

// use crate::client::MonzoClientError;

#[derive(Debug, Error)]
pub enum AppErrors {
    // -- General error
    #[error("Error: {0}")]
    Error(String),

    #[error("Can't set tracing Global Defafault")]
    SetGlobalDefaultError(#[from] tracing::subscriber::SetGlobalDefaultError),

    #[error("Can't set the logger")]
    SetLoggerError(#[from] tracing_log::log::SetLoggerError),

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

    #[error("Reqwest error: {0}")]
    ReqwestError(String),

    #[error("Server error")]
    ServerError,

    #[error("Invalid header value {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    // -- File error
    #[error("Failed to open file")]
    FileError(#[from] std::io::Error),

    #[error("Failed to deserialise toml")]
    TomlError(#[from] toml::ser::Error),

    #[error("Configuration error")]
    ConfigurationError(#[from] config::ConfigError),

    // -- Database error
    #[error("Query error")]
    QueryError(#[from] sqlx::Error),

    #[error("Query error {0}")]
    Duplicate(String),

    #[error("Database error")]
    DbError(String),

    #[error("Migration error")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    // -- Command error
    #[error("Command aborted")]
    AbortError,

    #[error("Currency not found: {0}")]
    CurrencyNotFound(String),

    #[error("Input error")]
    InputError(#[from] dialoguer::Error),
}

// Implementing From<reqwest::Error> for MyError
impl From<reqwest::Error> for AppErrors {
    fn from(error: reqwest::Error) -> Self {
        AppErrors::ReqwestError(error.to_string())
    }
}
