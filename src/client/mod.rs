#![allow(dead_code)]
#![allow(unused_variables)]

mod accounts;
mod balance;
mod transactions;
mod whoami;

use core::fmt;

use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Error as ReqwestError;
use serde::Deserialize;
use thiserror::Error;

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
    auth_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum MonzoClientError {
    #[error("Authorisation failure: {0}")]
    AuthorisationFailure(#[from] ErrorJson),
    #[error("Network request failed: {0}")]
    ReqwestError(#[from] ReqwestError),
}

impl MonzoClient {
    pub fn new(access_token: String) -> Result<Self, MonzoClientError> {
        let base_url = "https://api.monzo.com/".to_string();
        let auth_url = "https://auth.monzo.com/".to_string();
        let mut headers = HeaderMap::new();
        let auth_header_value = format!("Bearer {}", access_token);
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&auth_header_value).expect("Failed to create header value"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build client");

        Ok(MonzoClient {
            base_url,
            auth_url,
            client,
        })
    }
}
