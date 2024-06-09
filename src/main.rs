use std::fs::File;

use anyhow::Error;
use clap::Parser;
use cli::{command, Cli, Commands};
use configuration::get_configuration;

mod cli;
mod configuration;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Auth {} => {
            let tokens = command::auth().await?;
            let mut config = get_configuration()?;
            config.access_tokens = tokens;
            let file = File::create("configuration.yaml")?;
            serde_yaml::to_writer(file, &config)?;
        }
        Commands::Reset {} => {
            command::reset().await;
        }
    }

    Ok(())
}
