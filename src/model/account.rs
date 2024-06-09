//! Models for the account endpoint

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Accounts {
    pub accounts: Vec<Account>,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub closed: bool,
    pub created: DateTime<Utc>,
    pub description: String,
    pub owner_type: String,
    pub account_number: String,
    pub sort_code: String,
}
