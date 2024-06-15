use clap::Parser;
use colored::Colorize;

use monzo::{
    cli::{command, Cli, Commands},
    configuration::get_config,
    error::AppErrors as Error,
    model::DatabasePool,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = get_subscriber("monzo".into(), "error".into(), std::io::stdout);
    init_subscriber(subscriber)?;

    let configuration = get_config().expect("Failed to read configuration.");

    let pool = DatabasePool::new_from_config(configuration.clone()).await?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Balances {} => match command::balances().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Update { all, days } => {
            let mut before = chrono::Utc::now().naive_utc();
            let mut since =
                before - chrono::Duration::days(configuration.clone().default_days_to_update);

            if *all {
                since = configuration.clone().start_date;
                before = chrono::Utc::now().naive_utc();
            } else if let Some(days) = days {
                since = before - chrono::Duration::days(*days);
            }

            match command::update(pool, &since, &before).await {
                Ok(_) => {}
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Bean {} => match command::beancount(pool).await {
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
            Err(e) => eprintln!("{} Failed to reset the database {}", "ERROR:".red(), e),
        },
    }

    Ok(())
}
