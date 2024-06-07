use axum::{routing::get, Router};

use anyhow::Error;
use monzo::{
    auth::{oauth_callback, open_login_page},
    configuration::get_configuration,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Open the browser with login page
    let config = get_configuration().unwrap();
    let oath_credentials = config.oath_credentials;
    open_login_page(&oath_credentials.client_id, &oath_credentials.redirect_uri);

    // Create handler
    let app = Router::new().route("/oauth/callback", get(oauth_callback));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
