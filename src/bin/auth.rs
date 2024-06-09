// //! Handles obtaining and refreshing access tokens.

// use std::{collections::HashMap, fs::File};

// use anyhow::Error;
// use axum::{extract::Query, response::Html, routing::get, Router};
// use reqwest::Response;
// use serde::Deserialize;
// use tokio::time::{sleep, Duration};
// use tokio_util::sync::CancellationToken;
// use url::Url;
// use uuid::Uuid;

// use monzo::configuration::{get_configuration, AccessTokens, OathCredentials};

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     // Create server
//     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//         .await
//         .unwrap();
//     let app = Router::new().route("/oauth/callback", get(oauth_callback));

//     let token = CancellationToken::new();
//     let cloned_token = token.clone();

//     let _server_handle = tokio::task::spawn(async move {
//         tokio::select! {
//             // Step 3: Using cloned token to listen to cancellation requests
//             _ = cloned_token.cancelled() => {
//                 // The token was cancelled, task can shut down
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
//                 // Long work has completed
//             }
//         }
//         let _ = axum::serve(listener, app).await;
//     });

//     let config = get_configuration().unwrap();
//     open_login_page(
//         &config.oath_credentials.client_id,
//         &config.oath_credentials.redirect_uri,
//     )
//     .await;

//     sleep(Duration::from_secs(60)).await;
//     // loop {}
//     //
//     Ok(())
// }

// // Structure for representing the authcode request response
// #[derive(Deserialize, Debug)]
// struct AuthCodeResponse {
//     code: String,
//     #[serde(rename = "state")]
//     _state: String,
// }

// async fn open_login_page(client_id: &str, redirect_uri: &str) {
//     let state = Uuid::new_v4().to_string();

//     let mut params = HashMap::new();
//     params.insert("client_id", client_id);
//     params.insert("redirect_uri", redirect_uri);
//     params.insert("response_type", "code");
//     params.insert("state", &state);

//     let base_url = "https://auth.monzo.com/";
//     let url = generate_url(base_url, &params);

//     webbrowser::open(&url).expect("Failed to open browser");
// }

// // Generate the login URL
// fn generate_url(base_url: &str, params: &HashMap<&str, &str>) -> String {
//     let mut url = Url::parse(base_url).expect("Invalid base URL");
//     for (key, value) in params {
//         url.query_pairs_mut().append_pair(key, value);
//     }
//     url.to_string()
// }

// // oath callback function - handles the auth code response
// pub async fn oauth_callback(Query(params): Query<AuthCodeResponse>) -> Html<String> {
//     match exchange_auth_code_for_access_token(&params).await {
//         Ok(_) => "Successfully exchanged auth code for access token"
//             .to_string()
//             .into(),
//         Err(e) => format!("Error getting access token: {}", e).into(),
//     }
// }

// // Exchange the auth code for an access token
// async fn exchange_auth_code_for_access_token(params: &AuthCodeResponse) -> Result<(), Error> {
//     let response = submit_access_token_request(params).await?;
//     match response.status().is_success() {
//         true => {
//             let access_tokens = response.json::<AccessTokens>().await?;
//             save_access_tokens(access_tokens)?;
//             Ok(())
//         }
//         false => Err(anyhow::anyhow!(
//             "Failed to exchange auth code for access token"
//         )),
//     }
// }

// async fn submit_access_token_request(params: &AuthCodeResponse) -> Result<Response, Error> {
//     let config = get_configuration().unwrap();

//     let url = "https://api.monzo.com/oauth2/token";
//     let code = params.code.clone();
//     let params = build_form(&config.oath_credentials, &code);

//     let client = reqwest::Client::new();
//     let response = client.post(url).form(&params).send().await?;

//     Ok(response)
// }

// // Build the form for the access token request
// fn build_form<'a>(
//     oath_credentials: &'a OathCredentials,
//     code: &'a str,
// ) -> HashMap<&'a str, &'a str> {
//     let mut params = HashMap::new();
//     params.insert("grant_type", "authorization_code");
//     params.insert("client_id", &oath_credentials.client_id);
//     params.insert("client_secret", &oath_credentials.client_secret);
//     params.insert("redirect_uri", &oath_credentials.redirect_uri);
//     params.insert("code", code);

//     params
// }

// // Save the updated access tokens back to configuration
// fn save_access_tokens(access_tokens: AccessTokens) -> Result<(), Error> {
//     let mut config = get_configuration()?;
//     config.access_tokens = access_tokens;
//     let file = File::create("configuration.yaml")?;
//     serde_yaml::to_writer(file, &config)?;

//     Ok(())
// }
