//! Contains the `Directive` enum and its implementation

use chrono::NaiveDate;

use super::{AssetAccount, Transaction as BeanTransaction};

type Comment = String;

/// Represents a Beancount directive
#[derive(Debug)]
pub enum Directive {
    Comment(String),
    Open(NaiveDate, AssetAccount, Option<Comment>),
    Close(NaiveDate, AssetAccount, Option<Comment>),
    Transaction(BeanTransaction),
    Balance(NaiveDate, AssetAccount),
}

impl Directive {
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let account_width = 40;
        match self {
            Directive::Comment(comment) => format!("\n* {}\n", comment),
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
            Directive::Transaction(transaction) => {
                format!("{}\n", transaction.to_formatted_string())
            }
            Directive::Balance(_date, _account) => {
                todo!()
            }
        }
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::beancount::AccountType;

    use super::*;

    #[test]
    fn open_directive() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = AssetAccount {
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
        let account = AssetAccount {
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
        let account = AssetAccount {
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
        let account = AssetAccount {
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
}
