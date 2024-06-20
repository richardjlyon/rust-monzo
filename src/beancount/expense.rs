use core::fmt;

use super::AccountType;
use convert_case::{Case, Casing};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Expense {
    pub(crate) account_type: AccountType,
    pub(crate) category: String,
    pub(crate) sub_category: Option<String>,
}

impl fmt::Display for Expense {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sub_category = match &self.sub_category {
            Some(sub_category) => format!(":{}", sub_category.to_case(Case::Pascal)),
            None => String::new(),
        };
        write!(
            f,
            "{}{}{}",
            format!("{}", self.account_type),
            format!(":{}", self.category.to_case(Case::Upper)),
            format!("{}", sub_category),
        )
    }
}
