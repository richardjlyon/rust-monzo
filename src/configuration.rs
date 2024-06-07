use crate::client::auth::OathCredentials;
use anyhow::{Error, Result};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub oath_credentials: OathCredentials,
}

pub fn get_configuration() -> Result<Settings, Error> {
    // Initialise our configuration reader
    let settings = config::Config::builder()
        // Add configuration values from a file named `configuration.yaml`.
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()
        .expect("Failed to build configuration."); // FIXME map error
    Ok(settings.try_deserialize::<Settings>()?)
}

#[cfg(test)]
mod test {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn get_configuration_works() {
        let config = get_configuration().expect("Failed to read configuration.");

        assert!(config.oath_credentials.client_id.len() > 0);
    }
}
