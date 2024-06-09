#![allow(dead_code)]
#![allow(unused_variables)]

use crate::error::AppError as Error;
use core::fmt;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Error as ReqwestError, Response};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::configuration::get_configuration;
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

pub struct MonzoClient {
    base_url: String,
    client: reqwest::Client,
}

impl MonzoClient {
    pub fn new() -> Result<Self, Error> {
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

    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
        if response.status().is_success() {
            let result = response.json::<T>().await?;
            Ok(result)
        } else {
            let error_json = response.json::<ErrorJson>().await?;
            // Err(AnyError::msg(format!("Error: {:?}", error_json)))
            Err(Error::HandlerError)
        }
    }
}
