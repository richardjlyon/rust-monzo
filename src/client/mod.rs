#![allow(dead_code)]
#![allow(unused_variables)]

use crate::error::AppErrors as Error;
use core::fmt;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tracing_log::log::{error, info};

use crate::configuration::get_config;
// use crate::error::AppError as Error;

mod accounts;
mod balance;
mod pots;
pub mod transactions;
mod whoami;

#[derive(Debug, Deserialize, thiserror::Error)]
pub struct ErrorJson {
    code: String,
    message: String,
}

// Implement `fmt::Display` trait for `ErrorJson`.
impl fmt::Display for ErrorJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, message: {}", self.code, self.message)
    }
}

pub struct Monzo {
    base_url: String,
    client: reqwest::Client,
}

impl Monzo {
    /// Create a new Monzo client
    ///
    /// # Errors
    /// Will return an error if the auth header can't be created or the client can't be built.
    pub fn new() -> Result<Self, Error> {
        let base_url = "https://api.monzo.com/".to_string();
        let config = get_config()?;
        let mut headers = HeaderMap::new();
        let auth_header_value = format!("Bearer {}", config.access_tokens.access_token);
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&auth_header_value)?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Monzo { base_url, client })
    }

    #[tracing::instrument(name = "Handle response", skip(response))]
    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
        if response.status().is_success() {
            info!("Response is successful");
            let result = response.json::<T>().await?;
            Ok(result)
        } else {
            let error_json = response.json::<ErrorJson>().await?;
            // Err(AnyError::msg(format!("Error: {:?}", error_json)))
            error!("Response error: {:?}", error_json);
            Err(Error::HandlerError)
        }
    }
}
