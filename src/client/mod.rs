#![allow(dead_code)]
#![allow(unused_variables)]

mod accounts;
mod balance;
mod pots;
mod transactions;
mod whoami;

use core::fmt;

use anyhow::Error as AnyError;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Error as ReqwestError, Response};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use thiserror::Error;

use crate::configuration::get_configuration;

#[derive(Debug, Error)]
pub enum MonzoClientError {
    #[error("Authorisation failure: {0}")]
    AuthorisationFailure(#[from] ErrorJson),
    #[error("Network request failed: {0}")]
    ReqwestError(#[from] ReqwestError),
}

#[derive(Debug, Deserialize, Error)]
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

pub struct MonzoClient {
    base_url: String,
    client: reqwest::Client,
}

impl MonzoClient {
    pub fn new() -> Result<Self, MonzoClientError> {
        let base_url = "https://api.monzo.com/".to_string();
        let config = get_configuration().unwrap();
        let mut headers = HeaderMap::new();
        let auth_header_value = format!("Bearer {}", config.access_tokens.access_token);
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&auth_header_value).expect("Failed to create header value"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build client");

        Ok(MonzoClient { base_url, client })
    }

    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, AnyError> {
        if response.status().is_success() {
            let result = response.json::<T>().await?;
            Ok(result)
        } else {
            let error_json = response.json::<ErrorJson>().await?;
            Err(AnyError::msg(format!("Error: {:?}", error_json)))
        }
    }
}
