//! Balance related functions
//!
//! This module gets balance information from the Monzo API.

use super::MonzoClient;
use crate::error::AppError as Error;
use crate::model::balance::Balance;

impl MonzoClient {
    pub async fn balance(&self, account_id: &str) -> Result<Balance, Error> {
        let url = format!("{}balance?account_id={}", self.base_url, account_id);
        let response = self.client.get(&url).send().await?;
        let balance: Balance = Self::handle_response(response).await?;

        Ok(balance)
    }
}

#[cfg(test)]
mod test {
    use crate::tests::test::get_client;

    #[tokio::test]
    #[ignore]
    async fn balances_work() {
        let monzo = get_client();
        let accounts = monzo.accounts().await.unwrap();
        let account_id = &accounts[0].id;

        let balance = monzo.balance(account_id).await.unwrap();

        assert_eq!(balance.currency, "GBP");
    }
}
