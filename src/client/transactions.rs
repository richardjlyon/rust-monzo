use super::{ErrorJson, MonzoClient};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

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

        match response.status().is_success() {
            true => {
                let transactions = response.json::<Transactions>().await?;
                Ok(transactions.transactions)
            }
            false => {
                let error_json = response.json::<ErrorJson>().await?;
                Err(Error::msg(format!("Error: {:?}", error_json)))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tests::test::get_client;

    #[tokio::test]
    async fn transactions_work() {
        let monzo = get_client();
        let accounts = monzo.accounts().await.unwrap();
        let account_id = &accounts[0].id;

        let transactions = monzo.transactions(account_id).await.unwrap();

        assert_eq!(transactions[0].currency, "GBP");
    }
}
