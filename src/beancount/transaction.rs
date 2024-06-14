use chrono::NaiveDate;

use super::Account;

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
    use crate::beancount::account::AccountType;

    use super::*;

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
