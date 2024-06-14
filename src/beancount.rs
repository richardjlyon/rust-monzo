//! Beancount export
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in the database.

use capitalize::Capitalize;
use chrono::NaiveDate;
use core::fmt;
use serde::Deserialize;
use std::path::PathBuf;

use crate::error::AppErrors as Error;

/// A struct representing a Beancount configuration file on disk
#[derive(Debug, Deserialize)]
pub struct BeanSettings {
    pub beancount_filepath: PathBuf,
    pub start_date: NaiveDate,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Vec<Account>,
    pub equities: Vec<Account>,
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
    pub fn new() -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::File::new(
                "beancount.toml",
                config::FileFormat::Toml,
            ))
            .build()?;

        let settings = cfg.try_deserialize::<BeanSettings>()?;

        Ok(Beancount { settings })
    }

    /// Initialise the Beancount file from configuration settings
    pub fn init_from_config(&self) -> Result<(), Error> {
        // initiialse Equity accounts

        Ok(())
    }

    pub fn add_directive(&self, _directive: &Directive) {
        todo!()
    }

    pub fn add_transaction(&self, _transaction: &Transaction) {
        todo!()
    }

    pub fn to_string(&self) -> String {
        todo!()
    }
}

type Comment = String;

/// Represents a Beancount directive
#[derive(Debug)]
pub enum Directive {
    Open(NaiveDate, Account, Option<Comment>),
    Close(NaiveDate, Account, Option<Comment>),
    Balance(NaiveDate, Account),
    Comment(String),
}

impl Directive {
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let account_width = 40;
        match self {
            Directive::Open(date, account, comment) => {
                let currency = &account.currency;
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} open {:account_width$} {}",
                    comment,
                    date,
                    account.to_string(),
                    currency
                )
            }
            Directive::Close(date, account, comment) => {
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} close {:account_width$}",
                    comment,
                    date,
                    account.to_string(),
                )
            }
            Directive::Balance(_date, _account) => {
                todo!()
            }
            Directive::Comment(comment) => format!("\n* {}\n", comment),
        }
    }
}

/// Represents a Beancount account
#[derive(Debug, Deserialize, Clone)]
pub struct Account {
    pub(crate) account_type: AccountType,
    pub(crate) currency: String,
    pub(crate) provider: String,
    pub(crate) name: String,
}

// Implement Display for Account
impl fmt::Display for Account {
    // Remove space from the account name
    // e.g. "Assets:US:Bank of America:Checking" -> "Assets:US:

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.account_type,
            self.currency,
            self.provider,
            self.name.replace(' ', "").capitalize()
        )
    }
}

/// Represents permissable Beancount account types
#[derive(Debug, Clone, Deserialize, strum_macros::Display)]
pub enum AccountType {
    Assets,
    Liabilities,
    Equity,
    Income,
    Expense,
}

/// Represents a Beancount transaction
#[derive(Debug)]
pub struct Transaction {
    pub date: NaiveDate,
    pub description: String,
    pub postings: Postings,
}

impl Transaction {
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        format!(
            "{} * \"{}\"\n  {}\n  {}",
            self.date,
            self.postings.from.description,
            self.postings.from.to_formatted_string(),
            self.postings.to.to_formatted_string(),
        )
    }
}

/// Represents a Beancount double entry posting
#[derive(Debug)]
pub struct Postings {
    pub from: Posting,
    pub to: Posting,
}

/// represents a Beancount posting
#[derive(Debug)]
pub struct Posting {
    pub account: Account,
    pub amount: f64,
    pub currency: String,
    pub description: String,
}

// Implement Display for Account
impl Posting {
    fn to_formatted_string(&self) -> String {
        format!(
            "{:30} {:>10} {}",
            self.account.to_string(),
            self.amount.to_string(),
            self.currency,
        )
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_directive() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            currency: "GBP".to_string(),
            provider: "Monzo".to_string(),
            name: "Personal".to_string(),
        };
        // Act
        let directive = Directive::Open(date, account, None);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "2024-06-13 open Assets:GBP:Monzo:Personal                GBP"
        );
    }

    #[test]
    fn open_directive_comment() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            currency: "GBP".to_string(),
            provider: "Monzo".to_string(),
            name: "Personal".to_string(),
        };
        let comment = Some("Initial Deposit".to_string());
        // Act
        let directive = Directive::Open(date, account, comment);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "; Initial Deposit.\n2024-06-13 open Assets:GBP:Monzo:Personal                GBP"
        );
    }

    #[test]
    fn close_directive() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            currency: "GBP".to_string(),
            provider: "Monzo".to_string(),
            name: "Personal".to_string(),
        };
        // Act
        let directive = Directive::Close(date, account, None);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "2024-06-13 close Assets:GBP:Monzo:Personal               "
        );
    }

    #[test]
    fn close_directive_comment() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            currency: "GBP".to_string(),
            provider: "Monzo".to_string(),
            name: "Personal".to_string(),
        };
        let comment = Some("To Close".to_string());
        // Act
        let directive = Directive::Close(date, account, comment);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "; To Close.\n2024-06-13 close Assets:GBP:Monzo:Personal               "
        );
    }

    #[test]
    fn transaction_formatted() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let from_account = Account {
            account_type: AccountType::Assets,
            currency: "GBP".to_string(),
            provider: "Monzo".to_string(),
            name: "Personal".to_string(),
        };
        let from_posting = Posting {
            account: from_account,
            amount: 100.0,
            currency: "GBP".to_string(),
            description: "ONLINE PAYMENT - THANK YOU".to_string(),
        };
        let to_account = Account {
            account_type: AccountType::Liabilities,
            currency: "GBP".to_string(),
            provider: "Amex".to_string(),
            name: "Platinum".to_string(),
        };
        let to_posting = Posting {
            account: to_account,
            amount: -100.0,
            currency: "GBP".to_string(),
            description: "AMEX PAYMENT ACH PAYMENT".to_string(),
        };
        let postings = Postings {
            from: from_posting,
            to: to_posting,
        };
        let transaction = Transaction {
            date,
            description: "Yacht purchase".to_string(),
            postings,
        };
        let expected = r#"2024-06-13 * "ONLINE PAYMENT - THANK YOU"
  Assets:GBP:Monzo:Personal             100 GBP
  Liabilities:GBP:Amex:Platinum        -100 GBP"#;

        // Act
        let transaction_string = transaction.to_formatted_string();
        println!("{}", transaction_string);
        // Assert
        assert_eq!(transaction_string, expected);
    }
}
