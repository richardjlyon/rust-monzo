use std::collections::HashMap;

use super::MonzoClient;
use anyhow::{Error, Result};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
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
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub created: Option<DateTime<Utc>>,
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
    id: String,
    name: String,
    category: String,
    logo: Option<String>,
    address: Address,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    short_formatted: String,
    formatted: String,
    city: String,
    latitude: f64,
    longitude: f64,
    address: String,
    region: String,
    country: String,
    postcode: String,
}

#[derive(Deserialize, Debug)]
pub struct Categories {
    #[serde(flatten)]
    fields: HashMap<String, i32>,
}

#[derive(Deserialize, Debug)]
pub struct Attachment {
    id: String,
    external_id: String,
    file_url: String,
    file_type: String,
    created: DateTime<Utc>,
}

impl MonzoClient {
    /// Get maximum of [limit] transactions for the given account ID within the given date range
    /// Note: This will expand the merchant field for each transaction
    pub async fn transactions(
        &self,
        account_id: &str,
        since: DateTime<Utc>,
        before: DateTime<Utc>,
        limit: Option<u32>,
    ) -> Result<Vec<Transaction>, Error> {
        let since = since.to_rfc3339_opts(SecondsFormat::Secs, true);
        let before = before.to_rfc3339_opts(SecondsFormat::Secs, true);
        let limit = match limit {
            Some(l) => l,
            None => 100,
        };

        let url = format!(
            "{}transactions?account_id={}&since={}&before={}&limit={}&expand[]=merchant",
            self.base_url, account_id, since, before, limit
        );

        let response = self.client.get(&url).send().await?;
        let transactions: Transactions = Self::handle_response(response).await?;

        Ok(transactions.transactions)
    }
}

// Generate a date range for the given year and month
// Returns a tuple of (since, before) DateTime<Utc> to work with the Monzo API transactions endpoint
fn make_date_range(year: i32, month: u32) -> (DateTime<Utc>, DateTime<Utc>) {
    let length_seconds = 60 * 60 * 24 * num_days_in_month(year, month);

    let since = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();
    let before = since + chrono::Duration::seconds(length_seconds as i64 + 1);

    (since, before)
}

// Compute the number of days in a month for the given year and month, acocunting for leap years
fn num_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month"),
    }
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

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};

    use crate::tests::test::get_client;

    #[tokio::test]
    async fn transactions_work() {
        let monzo = get_client();
        let account_id = "acc_0000AdNaq81vwtbTBedL06";
        let (since, before) = super::make_date_range(2024, 5);
        let transactions = monzo
            .transactions(account_id, since, before, None)
            .await
            .unwrap();

        assert!(transactions.len() > 0);
    }

    #[test]
    fn date_range_works() {
        let (since, before) = super::make_date_range(2024, 5);

        let since_expected = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();
        let before_expected = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 1).unwrap();

        assert_eq!(since, since_expected);
        assert_eq!(before, before_expected);
    }
}
