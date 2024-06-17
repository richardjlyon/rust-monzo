//! Represents a Beancount account

use core::fmt;

use convert_case::{Case, Casing};
use serde::Deserialize;

/// Represents permissable Beancount account types
#[derive(Debug, Clone, Deserialize, strum_macros::Display)]
pub enum AccountType {
    Assets,
    Liabilities,
    Equities,
    Income,
    Expense,
}

/// Represents a Beancount account
/// e.g. `Assets:GBP:Monzo:Personal`
#[derive(Debug, Deserialize, Clone)]
pub struct Account {
    pub(crate) account_type: AccountType,
    pub(crate) currency: String,
    pub(crate) account_name: Option<String>,
    pub(crate) name: String,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let account_name = match &self.account_name {
            Some(name) => format!("{}:", name),
            None => String::new(),
        };
        write!(
            f,
            "{}{}{}{}",
            format!("{}:", self.account_type),
            format!("{}:", self.currency),
            account_name,
            format!("{}", self.name.to_case(Case::Pascal)),
        )
    }
}
