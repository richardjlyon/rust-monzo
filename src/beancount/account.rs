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

#[derive(Debug, Deserialize, Clone)]
pub struct LiabilityAccount {
    pub(crate) account_type: AccountType,
    pub(crate) currency: String,
    pub(crate) provider: String,
    pub(crate) category: String,
}

/// Represents a Beancount account
#[derive(Debug, Deserialize, Clone)]
pub struct AssetAccount {
    pub(crate) account_type: AccountType,
    pub(crate) currency: String,
    pub(crate) provider: String,
    pub(crate) name: String,
}

// Implement Display for a Liability
// e.g. `Liabilities:GBP:Monzo:Bills`
impl fmt::Display for LiabilityAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.account_type,
            self.currency,
            self.provider.to_case(Case::Pascal),
            self.category.to_case(Case::Pascal)
        )
    }
}

// Implement Display for an Asset
// e.g. `Assets:GBP:Monzo:Personal`
impl fmt::Display for AssetAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.account_type,
            self.currency,
            self.provider.to_case(Case::Pascal),
            self.name.replace(' ', "").to_case(Case::Title)
        )
    }
}
