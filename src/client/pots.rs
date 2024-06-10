//! Pots related functions
//!
//! This module gets pot information from the Monzo API.

use super::MonzoClient;
use crate::error::AppError as Error;
use crate::model::pots::{Pot, Pots};

impl MonzoClient {
    /// Get all pots that are not deleted for a given account
    pub async fn pots(&self, account_id: &str) -> Result<Vec<Pot>, Error> {
        let url = format!("{}pots?current_account_id={}", self.base_url, account_id);
        let response = self.client.get(&url).send().await?;
        let pots: Pots = Self::handle_response(response).await?;

        Ok(pots.pots)
    }
}

#[cfg(test)]
mod test {

    use crate::tests::test::get_client;

    #[tokio::test]
    #[ignore]
    async fn pots_work() {
        let monzo = get_client();
        let pots = monzo.pots("acc_0000AdNaq81vwtbTBedL06").await.unwrap();

        assert!(pots.len() > 0);
    }
}
