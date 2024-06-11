use serde::{Deserialize, Serialize};

use crate::error::AppErrors as Error;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub oath_credentials: OathCredentials,
    pub access_tokens: AccessTokens,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub database_path: String,
    pub max_connections: u32,
}

/// Structure for representing the components of the Oath client
#[derive(Debug, Serialize, Deserialize)]
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
        // Add configuration values from a file named `configuration.yaml`.
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;
    Ok(settings.try_deserialize::<Settings>()?)
}
