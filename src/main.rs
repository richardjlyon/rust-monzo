use clap::Parser;
use colored::Colorize;

use monzo_cli::{
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
            let end_date;
            let start_date;
            let config_start_date = configuration.start_date;
            let config_days_to_update = configuration.default_days_to_update;

            if *all {
                end_date = chrono::Utc::now().naive_utc();
                start_date = config_start_date;
            } else if let Some(days) = days {
                end_date = chrono::Utc::now().naive_utc();
                start_date = end_date - chrono::Duration::days(*days);
            } else {
                end_date = chrono::Utc::now().naive_utc();
                start_date = end_date - chrono::Duration::days(config_days_to_update);
            }

            match command::update(pool, start_date, end_date).await {
                Ok(_) => return Ok(()),
                Err(e) => return Err(Error::Error(e.to_string())),
            }
        }
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
