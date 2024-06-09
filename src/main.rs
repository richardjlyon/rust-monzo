use anyhow::Error;
use clap::Parser;
use cli::{command, Cli, Commands};

mod cli;
mod configuration;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
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
