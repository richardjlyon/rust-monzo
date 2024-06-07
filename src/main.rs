#![allow(dead_code)]
#![allow(unused_variables)]

use std::{collections::HashMap, fs::File};

use axum::{extract::Query, response::Html, routing::get, Router};

use anyhow::Error;
use monzo::{client::auth::OathCredentials, configuration::get_configuration};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::io::Write;
use url::Url;
use uuid::Uuid;
use webbrowser;

// Structure for representing the authcode request response
#[derive(Deserialize, Debug)]
struct AuthCodeResponse {
    code: String,
    state: String,
}

/// Structure for representing the components of the access token request response
#[derive(Debug, Deserialize, Serialize)]
pub struct AccessToken {
    pub access_token: String,
    pub client_id: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub token_type: String,
    pub user_id: String,
}

// Generate the login URL
fn generate_url(base_url: &str, params: &HashMap<&str, &str>) -> String {
    let mut url = Url::parse(base_url).expect("Invalid base URL");
    for (key, value) in params {
        url.query_pairs_mut().append_pair(key, value);
    }
    url.to_string()
}

pub fn open_login_page(client_id: &str, redirect_uri: &str) {
    let client = reqwest::Client::new();

    let url = "https://auth.monzo.com/";
    let state = Uuid::new_v4().to_string();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("response_type", "code");
    params.insert("state", &state);

    let url = generate_url(&url, &params);

    webbrowser::open(&url).expect("Failed to open browser");
}

// Build the form for the access token request
fn build_form<'a>(
    oath_credentials: &'a OathCredentials,
    code: &'a str,
) -> HashMap<&'a str, &'a str> {
    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", &&oath_credentials.client_id);
    params.insert("client_secret", &&oath_credentials.client_secret);
    params.insert("redirect_uri", &&oath_credentials.redirect_uri);
    params.insert("code", &code);

    params
}

async fn submit_access_token_request(params: &AuthCodeResponse) -> Result<Response, Error> {
    let config = get_configuration().unwrap();

    let url = "https://api.monzo.com/oauth2/token";
    let oath_credentials = config.oath_credentials.clone();
    let code = params.code.clone();
    let params = build_form(&oath_credentials, &code);

    let client = reqwest::Client::new();
    let response = client.post(url).form(&params).send().await?;

    Ok(response)
}

// Exchange the auth code for an access token
async fn exchange_auth_code_for_access_token(params: &AuthCodeResponse) -> Result<(), Error> {
    let response = submit_access_token_request(&params).await.unwrap();
    match response.status().is_success() {
        true => {
            let access_token = response.json::<AccessToken>().await.unwrap();
            let file = File::create("auth.yaml")?;
            serde_yaml::to_writer(file, &access_token)?;
            Ok(())
        }
        false => Err(anyhow::anyhow!(
            "Failed to exchange auth code for access token"
        )),
    }
}

async fn oauth_callback(Query(params): Query<AuthCodeResponse>) -> Html<String> {
    let _ = exchange_auth_code_for_access_token(&params).await;

    format!(
        "Received OAuth callback with code: {} and state: {}",
        params.code, params.state
    )
    .into()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Open the browser with login page
    let config = get_configuration().unwrap();
    let oath_credentials = config.oath_credentials;
    let auth_code = open_login_page(&oath_credentials.client_id, &oath_credentials.redirect_uri);

    // Create handler
    let app = Router::new().route("/oauth/callback", get(oauth_callback));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
