use super::{ErrorJson, MonzoClient};
use anyhow::{Error, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Balance {
    pub balance: i64,
    pub total_balance: i64,
    pub currency: String,
    pub spend_today: i64,
}

impl MonzoClient {
    pub async fn balance(&self, account_id: &str) -> Result<Balance, Error> {
        let url = format!("{}balance?account_id={}", self.base_url, account_id);
        let response = self.client.get(&url).send().await?;

        match response.status().is_success() {
            true => {
                let success_json = response.json::<Balance>().await?;
                Ok(success_json)
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
    async fn balances_work() {
        let monzo = get_client();
        let accounts = monzo.accounts().await.unwrap();
        let account_id = &accounts[0].id;

        let balance = monzo.balance(account_id).await.unwrap();

        assert_eq!(balance.currency, "GBP");
    }
}
