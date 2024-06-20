//! Represents a Beancount account
//!

use core::fmt;

use convert_case::{Case, Casing};
use serde::Deserialize;

/// Represents permissable Beancount account types
#[derive(Debug, Clone, Deserialize, PartialEq, strum_macros::Display)]
pub enum AccountType {
    Assets,
    Liabilities,
    Income,
    Expenses,
    Equity,
}

/// Represents a Beancount account
///
/// [Assets][Currency][AccountName][AssetName] e.g. Assets:GBP:Personal:Savings
/// [Liabilities][Currency][AccountNAme][LiabilityName] e.g. Liabilities:GBP:CreditCard:Amex
/// [Equities][Currency][AccountName][EquityName] e.g. Equities:GBP:OpeningBalances
/// [Income][Currency][AccountName][IncomeName] e.g. Income:GBP:Salary:BP
/// [Expenses][Currency][AccountName][ExpenseName] e.g. Expenses:GBP:Personal:Groceries
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Account {
    pub(crate) account_type: AccountType,
    pub(crate) country: String,
    pub(crate) institution: String,
    pub(crate) account: String,
    pub(crate) sub_account: Option<String>,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label = match &self.sub_account {
            Some(label) => format!(":{}", label.to_case(Case::Pascal)),
            None => String::new(),
        };
        write!(
            f,
            "{}{}{}{}",
            format!("{}", self.account_type),
            format!(":{}", self.country.to_case(Case::Upper)),
            format!(":{}", self.account.to_case(Case::Pascal)),
            format!("{}", label),
        )
    }
}
