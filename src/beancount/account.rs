//! Represents a Beancount account

use core::fmt;

use capitalize::Capitalize;
use serde::Deserialize;

/// Represents a Beancount account
#[derive(Debug, Deserialize, Clone)]
pub struct Account {
    pub(crate) account_type: AccountType,
    pub(crate) currency: String,
    pub(crate) provider: String,
    pub(crate) name: String,
}

/// Represents permissable Beancount account types
#[derive(Debug, Clone, Deserialize, strum_macros::Display)]
pub enum AccountType {
    Assets,
    Liabilities,
    Equities,
    Income,
    Expense,
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
