use chrono::NaiveDate;

use super::{Account, AccountType};

/// Represents a Beancount transaction
#[derive(Debug)]
pub struct Transaction {
    pub date: NaiveDate,
    pub comment: Option<String>,
    pub notes: String,
    pub postings: Postings,
}

/// Represents a Beancount double entry posting
#[derive(Debug, Clone)]
pub struct Postings {
    pub to: Posting,
    pub from: Posting,
}

/// represents a Beancount Liability posting
#[derive(Debug, Clone)]
pub struct Posting {
    pub account: Account,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
}

impl Transaction {
    #[must_use]

    pub fn to_formatted_string(&self) -> String {
        let comment = match &self.comment {
            Some(s) if s.trim().is_empty() => String::new(),
            Some(d) => format!("; {}\n", d),
            None => String::new(),
        };

        format!(
            "{}{} * \"{}\"\n  {}\n  {}\n",
            comment,
            self.date,
            self.notes,
            self.postings.to.to_formatted_string(),
            self.postings.from.to_formatted_string(),
        )
    }
}

// FIXME: Formatting is conditional on self.account.account_type
impl Posting {
    fn to_formatted_string(&self) -> String {
        let amount = self.amount / 100.0;

        match self.account.account_type {
            AccountType::Assets => {
                format!(
                    "{:<50} {:>10.2} {}",
                    self.account.to_string(),
                    amount,
                    self.currency,
                )
            }
            AccountType::Liabilities => {
                format!(
                    "{:<50} {:>10.2} {}",
                    self.account.to_string(),
                    amount,
                    self.currency,
                )
            }
            AccountType::Income => {
                format!(
                    "{:<50} {:>10.2} {}",
                    self.account.to_string(),
                    amount,
                    self.currency,
                )
            }
            AccountType::Expenses => {
                format!(
                    "{:<50} {:>10.2} {}",
                    self.account.to_string(),
                    amount,
                    self.currency,
                )
            }
            AccountType::Equity => {
                format!(
                    "{:<50} {:>10.2} {}",
                    self.account.to_string(),
                    amount,
                    self.currency,
                )
            }
        }
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::beancount::account::AccountType;

    use super::*;

    #[test]
    fn transaction_formatted() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();

        let liability_account = Account {
            account_type: AccountType::Liabilities,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Groceries".to_string(),
            sub_account: None,
        };

        let asset_account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
        };

        let liability_posting = Posting {
            account: liability_account,
            amount: -1000.0,
            currency: "GBP".to_string(),
            description: Some("AMEX PAYMENT ACH PAYMENT".to_string()),
        };

        let asset_posting = Posting {
            account: asset_account,
            amount: 1000.0,
            currency: "GBP".to_string(),
            description: None,
        };

        let postings = Postings {
            from: asset_posting,
            to: liability_posting,
        };
        let transaction = Transaction {
            comment: Some("ONLINE PAYMENT - THANK YOU".to_string()),
            date,
            notes: "Yacht purchase".to_string(),
            postings,
        };
        let expected = r#"; ONLINE PAYMENT - THANK YOU
2024-06-13 * "Yacht purchase"
  Liabilities:GBP:Groceries                              -10.00 GBP
  Assets:GBP:Personal                                     10.00 GBP
"#;

        // Act
        let transaction_string = transaction.to_formatted_string();

        // Assert
        println!("{}", transaction_string);
        assert_eq!(transaction_string, expected);
    }
}
