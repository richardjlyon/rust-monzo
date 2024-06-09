use anyhow::Error;
use axum::{routing::get, Router};
use clap::Parser;
use cli::{command, Cli, Commands};
use monzo::routes::oauth_callback;
use tokio_util::sync::CancellationToken;

mod cli;
mod configuration;

#[tokio::main]
async fn main() -> Result<(), Error> {
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

    let cli = Cli::parse();

    match &cli.command {
        Commands::Auth {} => {
            command::auth().await;
        }

        Commands::Reset {} => {
            command::reset().await;
        }
    }

    Ok(())
}
