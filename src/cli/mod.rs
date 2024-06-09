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
    /// (Re)authorise the application
    Auth {},

    /// Reset the database (WARNING: This will delete all data!)
    Reset {},
}
