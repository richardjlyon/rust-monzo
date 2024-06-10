//! Reset the database to its initial state.
//!
//! This command will delete all data from the database.
//!
use colored::Colorize;
use dialoguer::Confirm;
use std::env;
use std::path::Path;

use crate::{configuration::get_configuration, error::AppError as Error, model::DatabasePool};

pub async fn reset() -> Result<(), Error> {
    if !confirm_reset() {
        println!("Aborting");
        return Ok(());
    }

    let config = get_configuration().expect("Failed to read configuration.");

    let current_dir = env::current_dir()?;
    let file_path = current_dir.join(&config.database.database_path);

    if Path::new(&file_path).exists() {
        std::fs::remove_file(&file_path)?;
    }

    match DatabasePool::new_from_config(config).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::DbError(e.to_string())),
    }
}

fn confirm_reset() -> bool {
    println!("Resetting the database");
    println!(
        "{} {}",
        "WARNING".red(),
        "This destroys all data and cannot be undone".bold()
    );
    let confirmation = Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
        .unwrap();

    confirmation
}
