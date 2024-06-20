//! Contains the `Directive` enum and its implementation

use chrono::NaiveDate;
use convert_case::{Case, Casing};

use super::{equity::Equity, expense::Expense, Account, Transaction as BeanTransaction};

type Comment = String;

/// Represents a Beancount directive
#[derive(Debug)]
pub enum Directive {
    Comment(String),
    OpenAccount(NaiveDate, Account, Option<Comment>),
    OpenExpense(NaiveDate, Expense, Option<Comment>),
    OpenEquity(NaiveDate, Equity, Option<Comment>),
    Close(NaiveDate, Account, Option<Comment>),
    Transaction(BeanTransaction),
    Balance(NaiveDate, Account),
}

impl Directive {
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let account_width = 40;
        match self {
            Directive::Comment(comment) => format!("\n* {}\n\n", comment.to_case(Case::Title)),
            Directive::OpenAccount(date, account, comment) => {
                let currency = &account.country;
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} open {:account_width$} {}\n",
                    comment,
                    date,
                    account.to_string(),
                    currency
                )
            }
            Directive::OpenExpense(date, expense, comment) => {
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} open {:account_width$}\n",
                    comment,
                    date,
                    expense.to_string(),
                )
            }
            Directive::OpenEquity(date, equity, comment) => {
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} open {:account_width$}\n",
                    comment,
                    date,
                    equity.to_string(),
                )
            }
            Directive::Close(date, account, comment) => {
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} close {:account_width$}\n",
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
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
        };
        // Act
        let directive = Directive::OpenAccount(date, account, None);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "2024-06-13 open Assets:GBP:Personal                      GBP\n"
        );
    }

    #[test]
    fn open_directive_comment() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
        };
        let comment = Some("Initial Deposit".to_string());
        // Act
        let directive = Directive::OpenAccount(date, account, comment);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "; Initial Deposit.\n2024-06-13 open Assets:GBP:Personal                      GBP\n"
        );
    }

    #[test]
    fn close_directive() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
        };
        // Act
        let directive = Directive::Close(date, account, None);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "2024-06-13 close Assets:GBP:Personal                     \n"
        );
    }

    #[test]
    fn close_directive_comment() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
        };
        let comment = Some("To Close".to_string());
        // Act
        let directive = Directive::Close(date, account, comment);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "; To Close.\n2024-06-13 close Assets:GBP:Personal                     \n"
        );
    }
}
