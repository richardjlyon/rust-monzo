use super::MonzoClient;
// use anyhow::{Error, Result};
use anyhow::Error;
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

impl MonzoClient {
    pub async fn accounts(&self) -> Result<Vec<Account>, Error> {
        let url = format!("{}accounts", self.base_url);
        let response = self.client.get(&url).send().await?;
        let accounts: Accounts = Self::handle_response(response).await?;

        Ok(accounts.accounts)
    }
}

#[cfg(test)]
mod test {

    use crate::tests::test::get_client;

    #[tokio::test]
    #[ignore]
    async fn accounts_work() {
        // Arrange
        let monzo = get_client();
        // Act
        let accounts = monzo.accounts().await.unwrap();
        // Assert
        assert!(accounts.len() > 0);
    }
}
