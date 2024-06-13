//! Auth
//!
//! This command will obtain an access token from Monzo, exchange it
//! for an authorisation token, and persist it to the configuration file.

use std::collections::HashMap;
use std::io::Write;
use std::option::Option;
use std::sync::Arc;
use tokio::sync::watch;
use url::Url;
use uuid::Uuid;

use crate::configuration::{get_config, AccessTokens};
use crate::error::AppErrors as Error;
use crate::routes::oauth_callback;
use axum::{routing::get, Router};

#[derive(Clone)]
pub struct AuthorisationState {
    pub token_tx: Arc<watch::Sender<Option<AccessTokens>>>,
}

/// Authenticate with Monzo
///
/// # Errors
///
/// Will return errors if the configuration file does not exist or cannot be written to.
pub async fn auth() -> Result<(), Error> {
    let access_tokens = get_access_tokens().await?;

    let mut config = get_config()?;
    config.access_tokens = access_tokens;
    let mut file = std::fs::File::create("configuration.toml")?;
    let toml_string = toml::to_string_pretty(&config)?;
    file.write_all(toml_string.as_bytes())?;

    Ok(())
}

// Get the access tokens.
//
// This function will open the browser to the Monzo OAuth page and listen for the callback.
//
// Implementation note: We fire up a server to listen for the OAuth callback and implement a watch channel to allow
// it to signal when the access tokens are received.
async fn get_access_tokens() -> Result<AccessTokens, Error> {
    let config = get_config()?;

    // Create server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    let (token_tx, mut token_rx) = watch::channel(None);

    let state = AuthorisationState {
        token_tx: Arc::new(token_tx),
    };

    let app = Router::new()
        .route("/oauth/callback", get(oauth_callback))
        .with_state(state);

    tokio::select! {
        _ = async {axum::serve(listener, app).await } => {
            Err(Error::ServerError)
        },

        access_tokens = async {
            open_login_page(
                &config.oath_credentials.client_id,
                &config.oath_credentials.redirect_uri,
            );
            token_rx.wait_for(Option::is_some).await
        } => {
            access_tokens.map(|v| v.as_ref().expect("checked Some above").to_owned()).map_err(|e| Error::AccessTokenError(e.to_string()))
        }
    }
}

// Generate the login URL
fn generate_url(params: &HashMap<&str, &str>) -> String {
    let base_url = "https://auth.monzo.com/";
    let mut url = Url::parse(base_url).expect("Invalid base URL");
    for (key, value) in params {
        url.query_pairs_mut().append_pair(key, value);
    }
    url.to_string()
}

fn open_login_page(client_id: &str, redirect_uri: &str) {
    let state = Uuid::new_v4().to_string();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("response_type", "code");
    params.insert("state", &state);

    let url = generate_url(&params);

    webbrowser::open(&url).expect("Failed to open browser");
}
