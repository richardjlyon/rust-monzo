use axum::{
    extract::{Query, State},
    response::Html,
};
use reqwest::Response;
use serde::Deserialize;
use std::collections::HashMap;

use crate::error::AppErrors as Error;
use crate::{
    cli::command::auth::AuthorisationState,
    configuration::{get_config, AccessTokens, OathCredentials},
};

// Structure for representing the authcode request response
#[derive(Deserialize, Debug)]
pub struct AuthCodeResponse {
    pub code: String,
    #[serde(rename = "state")]
    _state: String,
}

// oath callback function - handles the auth code response
pub async fn oauth_callback(
    Query(params): Query<AuthCodeResponse>,
    State(state): State<AuthorisationState>,
) -> Html<String> {
    match exchange_auth_code_for_access_token(&params).await {
        Ok(token) => {
            _ = state.token_tx.send(Some(token));
            "success".to_string().into()
        }
        Err(e) => format!("Error getting access token: {e}").into(),
    }
}

async fn exchange_auth_code_for_access_token(
    params: &AuthCodeResponse,
) -> Result<AccessTokens, Error> {
    let response = submit_access_token_request(params).await?;
    if response.status().is_success() {
        Ok(response.json::<AccessTokens>().await?)
    } else {
        Err(Error::AuthCodeExchangeError)
    }
}

async fn submit_access_token_request(params: &AuthCodeResponse) -> Result<Response, Error> {
    let config = get_config()?;

    let url = "https://api.monzo.com/oauth2/token";
    let code = params.code.clone();
    let params = build_form(&config.oath_credentials, &code);

    let client = reqwest::Client::new();
    let response = client.post(url).form(&params).send().await?;

    Ok(response)
}

// Build the form for the access token request
fn build_form<'a>(
    oath_credentials: &'a OathCredentials,
    code: &'a str,
) -> HashMap<&'a str, &'a str> {
    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", &oath_credentials.client_id);
    params.insert("client_secret", &oath_credentials.client_secret);
    params.insert("redirect_uri", &oath_credentials.redirect_uri);
    params.insert("code", code);

    params
}
