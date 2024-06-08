use super::MonzoClient;
// use anyhow::{Error, Result};
use anyhow::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Accounts {
    pub accounts: Vec<Account>,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub closed: bool,
    pub created: String,
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
    async fn accounts_work() {
        let monzo = get_client();
        let accounts = monzo.accounts().await.unwrap();
        let account_id = &accounts[0].id;
        let transactions = monzo.transactions(account_id).await.unwrap();

        assert_eq!(transactions[0].currency, "GBP");
    }
}
