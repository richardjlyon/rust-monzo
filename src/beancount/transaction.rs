use chrono::NaiveDate;

use super::Account;

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
    pub liability_posting: LiabilityPosting,
    pub asset_posting: AssetPosting,
}

/// represents a Beancount Liability posting
#[derive(Debug, Clone)]
pub struct LiabilityPosting {
    pub account: Account,
    pub amount: f64,
    pub currency: String,
    pub description: String,
}

/// represents a Beancount Asset posting
#[derive(Debug, Clone)]
pub struct AssetPosting {
    pub account: Account,
    pub amount: f64,
    pub currency: String,
}

impl Transaction {
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let comment = match &self.comment {
            Some(s) if s.is_empty() => String::new(),
            Some(d) => format!("; {}\n", d),
            None => String::new(),
        };

        format!(
            "{}{} * \"{}\"\n  {}\n  {}",
            comment,
            self.date,
            self.notes,
            self.postings.liability_posting.to_formatted_string(),
            self.postings.asset_posting.to_formatted_string(),
        )
    }
}

// Implement Display for Liability Posting
// e.g. `Liabilities:GBP:Monzo:Bills         59.99 GBP`
impl LiabilityPosting {
    fn to_formatted_string(&self) -> String {
        let amount = self.amount / 100.0;
        format!(
            "{:50} {:>10.2} {}",
            self.account.to_string(),
            amount,
            self.currency,
        )
    }
}

// Implement Display for Asset Posting
// e.g. `Assets:GBP:Monzo:Personal         -59.99 GBP`
impl AssetPosting {
    fn to_formatted_string(&self) -> String {
        let amount = self.amount / 100.0;
        format!(
            "{:50} {:>10.2} {}",
            self.account.to_string(),
            amount,
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

        let liability_account = Account {
            account_type: AccountType::Liabilities,
            currency: "GBP".to_string(),
            name: "Groceries".to_string(),
        };

        let asset_account = AssetAccount {
            account_type: AccountType::Assets,
            currency: "GBP".to_string(),
            provider: "Monzo".to_string(),
            name: "Personal".to_string(),
        };

        let liability_posting = LiabilityPosting {
            account: liability_account,
            amount: -1000.0,
            currency: "GBP".to_string(),
            description: "AMEX PAYMENT ACH PAYMENT".to_string(),
        };

        let asset_posting = AssetPosting {
            account: asset_account,
            amount: 1000.0,
            currency: "GBP".to_string(),
        };

        let postings = Postings {
            asset_posting,
            liability_posting,
        };
        let transaction = Transaction {
            comment: Some("ONLINE PAYMENT - THANK YOU".to_string()),
            date,
            notes: "Yacht purchase".to_string(),
            postings,
        };
        let expected = r#"; ONLINE PAYMENT - THANK YOU
2024-06-13 * "Yacht purchase"
  Liabilities:GBP:Groceries          -10.00 GBP
  Assets:GBP:Monzo:Personal           10.00 GBP"#;

        // Act
        let transaction_string = transaction.to_formatted_string();

        // Assert
        println!("{}", transaction_string);
        assert_eq!(transaction_string, expected);
    }
}
