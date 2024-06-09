use clap::Parser;
use monzo::{
    cli::{command, Cli, Commands},
    configuration::get_configuration,
    error::AppError as Error,
    model::DatabasePool,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
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
        Commands::Reset {} => {
            command::reset().await;
        }
    }

    Ok(())
}
