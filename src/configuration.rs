use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppErrors as Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub start_date: DateTime<Utc>,
    pub default_days_to_update: i64,
    pub database: Database,
    pub oath_credentials: OathCredentials,
    pub access_tokens: AccessTokens,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    pub database_path: String,
    pub max_connections: u32,
}

/// Structure for representing the components of the Oath client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OathCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// Structure for representing the components of the access token request response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccessTokens {
    pub access_token: String,
    pub client_id: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub token_type: String,
    pub user_id: String,
}

/// Get the configuration from the configuration file
///
/// # Errors
/// Will return errors if the config can't be read or deserialised.
pub fn get_config() -> Result<Settings, Error> {
    // Initialise our configuration reader
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.toml",
            config::FileFormat::Toml,
        ))
        .build()?;
    Ok(settings.try_deserialize::<Settings>()?)
}
