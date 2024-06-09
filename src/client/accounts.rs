use std::collections::HashMap;

use crate::error::AppError as Error;
use crate::model::account::{Account, Accounts};

use super::MonzoClient;

impl MonzoClient {
    /// Get a list of accounts
    pub async fn accounts(&self) -> Result<Vec<Account>, Error> {
        let url = format!("{}accounts", self.base_url);
        let response = self.client.get(&url).send().await?;
        let accounts: Accounts = Self::handle_response(response).await?;

        Ok(accounts.accounts)
    }

    /// Generate a hash of account IDs and descriptions
    pub async fn account_description_from_id(&self) -> Result<HashMap<String, String>, Error> {
        let mut accounts = HashMap::new();
        for account in self.accounts().await? {
            accounts.insert(account.id, account.owner_type);
        }

        Ok(accounts)
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

    #[tokio::test]
    async fn account_hash_works() {
        // Arrange
        let monzo = get_client();
        // Act
        let companies = monzo.account_description_from_id().await.unwrap();
        // Assert
        println!("{:#?}", companies);
    }
}
