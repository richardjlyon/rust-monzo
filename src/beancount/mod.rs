//! Beancount export
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in the database.

mod account;
mod directive;
mod equity;
mod expense;
mod transaction;

use chrono::NaiveDate;
use equity::Equity;
use expense::Expense;

use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

use crate::error::AppErrors as Error;

pub use account::{Account, AccountType};
pub use directive::Directive;
pub use transaction::{Posting, Postings, Transaction};

/// A struct representing a Beancount file
pub struct Beancount {
    pub settings: BeanSettings,
}

/// A struct representing a Beancount configuration file on disk
#[derive(Debug, Deserialize)]
pub struct BeanSettings {
    pub beancount_filepath: PathBuf,
    pub start_date: NaiveDate,
    pub custom_categories: Option<HashMap<String, String>>,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
    pub expenses: Option<Vec<Expense>>,
    pub equity: Option<Vec<Equity>>,
}

impl Beancount {
    /// Create a new Beancount instance
    ///
    /// # Errors
    /// Will return an error if the configuration file cannot be read
    pub fn from_config() -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::File::new(
                "beancount.yaml",
                config::FileFormat::Yaml,
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

    // Iniitialise the file system
    // pub fn initialise_filesystem(&self) -> Result<(), Error> {
    //     let path = self.settings.beancount_filepath.clone();
    //     let parent = path.parent().ok_or(Error::PathError)?;
    //     std::fs::create_dir_all(parent)?;
    //     Ok(())
    // }
}
