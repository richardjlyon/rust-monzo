use core::fmt;

use super::AccountType;
use convert_case::{Case, Casing};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Equity {
    pub(crate) account_type: AccountType,
    pub(crate) account: String,
}

impl fmt::Display for Equity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            format!("{}", self.account_type),
            format!(":{}", self.account.to_case(Case::Pascal)),
        )
    }
}
