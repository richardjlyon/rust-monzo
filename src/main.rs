use anyhow::Error;
use clap::Parser;
use monzo::cli::{command, Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Balances {} => match command::balances().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Auth {} => match command::auth().await {
            Ok(_) => println!("Auth completed"),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Reset {} => {
            command::reset().await;
        }
    }

    Ok(())
}
