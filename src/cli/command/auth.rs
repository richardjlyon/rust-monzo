use std::collections::HashMap;

use axum::{routing::get, Router};

use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use url::Url;
use uuid::Uuid;

use crate::configuration::get_configuration;
use crate::routes::oauth_callback;

pub async fn auth() {
    let config = get_configuration().unwrap();

    // Create server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    let app = Router::new().route("/oauth/callback", get(oauth_callback));

    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let _server_handle = tokio::task::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token.cancelled() => {
                // The token was cancelled, task can shut down
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                // Long work has completed
            }
        }
        let _ = axum::serve(listener, app).await;
    });

    open_login_page(
        &config.oath_credentials.client_id,
        &config.oath_credentials.redirect_uri,
    )
    .await;

    sleep(Duration::from_secs(60)).await;
}

async fn open_login_page(client_id: &str, redirect_uri: &str) {
    let state = Uuid::new_v4().to_string();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("response_type", "code");
    params.insert("state", &state);

    let base_url = "https://auth.monzo.com/";
    let url = generate_url(base_url, &params);

    webbrowser::open(&url).expect("Failed to open browser");
}

// Generate the login URL
fn generate_url(base_url: &str, params: &HashMap<&str, &str>) -> String {
    let mut url = Url::parse(base_url).expect("Invalid base URL");
    for (key, value) in params {
        url.query_pairs_mut().append_pair(key, value);
    }
    url.to_string()
}
