//! Monzo App Command Line Interface

pub mod command;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Update transactions
    Update {
        /// Flag to update all transactions
        #[arg(short, long)]
        all: bool,

        /// Days to get (optional, defaults to configuration setting `default_days_to_update`)
        #[arg(short, long)]
        days: Option<i64>,
    },
    /// Account balances
    Balances {},
    /// (Re)authorise the application
    Auth {},
    /// Reset the database (WARNING: This will delete all data!)
    Reset {},
}
