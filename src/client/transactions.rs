use super::MonzoClient;
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    pub amount: i64,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub created: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub settled: Option<DateTime<Utc>>,
    pub currency: String,
    pub description: String,
    pub category: String,
    pub notes: String,
    pub merchant: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct DateRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

impl DateRange {
    pub fn new(from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        DateRange { from, to }
    }

    // default is 365 days ago from now
    pub fn default() -> Self {
        let now = Utc::now();
        let from = now - chrono::Duration::days(365);
        DateRange { from, to: now }
    }

    fn to_rfc3339(&self) -> (String, String) {
        (
            self.from.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            self.to.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        )
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

impl MonzoClient {
    pub async fn transactions(&self, account_id: &str) -> Result<Vec<Transaction>, Error> {
        let url = format!("{}transactions?account_id={}", self.base_url, account_id);
        let response = self.client.get(&url).send().await?;
        let transactions: Transactions = Self::handle_response(response).await?;

        Ok(transactions.transactions)
    }
}

#[cfg(test)]
mod test {
    use super::DateRange;
    use crate::tests::test::get_client;

    #[tokio::test]
    async fn transactions_work() {
        let monzo = get_client();
        let accounts = monzo.accounts().await.unwrap();
        let account_id = &accounts[0].id;

        println!("->> Account id: {}", account_id);

        let transactions = monzo.transactions(account_id).await.unwrap();

        println!("got {} transactions", transactions.len());

        assert_eq!(transactions[0].currency, "GBP");
    }

    #[tokio::test]
    async fn date_range_works() {
        let range = DateRange::default();
        println!("{:?}", range);
        let (from, to) = range.to_rfc3339();
        println!("from: {}, to: {}", from, to);
    }
}
