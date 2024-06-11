//! Reset the database to its initial state.
//!
//! This command will delete all data from the database.
//!
use colored::Colorize;
use dialoguer::Confirm;
use std::env;
use std::path::Path;

use crate::{configuration::get_config, error::AppErrors as Error, model::DatabasePool};

/// Reset the database to its initial state.
///
/// # Errors
/// Will return errors if the database file cannot be deleted or if the database pool cannot be created.
pub async fn reset() -> Result<DatabasePool, Error> {
    if !confirm_reset()? {
        return Err(Error::AbortError);
    }

    let config = get_config()?;

    let current_dir = env::current_dir()?;
    let file_path = current_dir.join(&config.database.database_path);

    if Path::new(&file_path).exists() {
        std::fs::remove_file(&file_path)?;
    }

    DatabasePool::new_from_config(config)
        .await
        .map_err(|e| Error::DbError(e.to_string()))
}

fn confirm_reset() -> Result<bool, Error> {
    println!("Resetting the database");
    println!(
        "{} {}",
        "WARNING".red(),
        "This destroys all data and cannot be undone".bold()
    );
    let confirmation = Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()?;

    Ok(confirmation)
}
