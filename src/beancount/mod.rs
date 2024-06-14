//! Beancount export
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in the database.

use chrono::NaiveDate;

use serde::Deserialize;
use std::path::PathBuf;

use crate::error::AppErrors as Error;

mod account;
mod directive;
mod transaction;

pub use account::{Account, AccountType};
pub use directive::Directive;
pub use transaction::{Posting, Postings, Transaction};

/// A struct representing a Beancount configuration file on disk
#[derive(Debug, Deserialize)]
pub struct BeanSettings {
    pub beancount_filepath: PathBuf,
    pub start_date: NaiveDate,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub equities: Option<Vec<Account>>,
}

/// A struct representing a Beancount file
pub struct Beancount {
    pub settings: BeanSettings,
}

impl Beancount {
    /// Create a new Beancount instance
    ///
    /// # Errors
    /// Will return an error if the configuration file cannot be read
    pub fn from_config() -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::File::new(
                "beancount.toml",
                config::FileFormat::Toml,
            ))
            .build()?;

        match cfg.try_deserialize::<BeanSettings>() {
            Ok(settings) => Ok(Beancount { settings }),
            Err(e) => {
                println!("{}", e.to_string());
                Err(Error::ConfigurationError(e))
            }
        }
    }
}
