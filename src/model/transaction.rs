//! Models for the transaction endpoint

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    pub dedupe_id: String,
    pub account_id: String,
    pub amount: i64,
    pub currency: String,
    pub local_amount: i64,
    pub local_currency: String,
    // #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub created: DateTime<Utc>,
    pub description: String,
    pub amount_is_pending: bool,
    pub merchant: Option<Merchant>,
    pub notes: String,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub settled: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub updated: Option<DateTime<Utc>>,
    pub category: String,
    pub categories: Categories,
    pub attachments: Option<Vec<Attachment>>,
}
#[derive(Deserialize, Debug)]
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub category: String,
    pub logo: Option<String>,
    pub address: Address,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    pub short_formatted: String,
    pub formatted: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub region: String,
    pub country: String,
    pub postcode: String,
}

#[derive(Deserialize, Debug)]
pub struct Categories {
    #[serde(flatten)]
    _fields: HashMap<String, i32>,
}

#[derive(Deserialize, Debug)]
pub struct Attachment {
    _id: String,
    _external_id: String,
    _file_url: String,
    _file_type: String,
    _created: DateTime<Utc>,
}

// Custom deserialization function for Option<DateTime<Utc>>
fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt.as_deref() {
        Some("") | None => Ok(None),
        Some(s) => match DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
            Err(_) => Err(serde::de::Error::custom(format!(
                "invalid date-time format: {}",
                s
            ))),
        },
    }
}
