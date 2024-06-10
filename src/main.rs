use clap::Parser;
use colored::Colorize;

use monzo::{
    cli::{command, Cli, Commands},
    configuration::get_configuration,
    error::AppError as Error,
    model::DatabasePool,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = get_subscriber("monzo".into(), "error".into(), || std::io::stdout());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = DatabasePool::new_from_config(configuration).await?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Balances {} => match command::balances().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Update {} => match command::update(connection_pool).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Auth {} => match command::auth().await {
            Ok(_) => println!("Auth completed"),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Reset {} => match command::reset().await {
            Ok(_) => println!("{}", "Database reset complete".green()),
            Err(Error::AbortError) => println!("{}", "Database reset aborted".yellow()),
            Err(e) => eprintln!(
                "{} {} {}",
                "ERROR:".red(),
                "Failed to reset the database",
                e.to_string()
            ),
        },
    }

    Ok(())
}
