//! Transaction related functions
//!
//! This module gets transaction information from the Monzo API.

use chrono::NaiveDateTime;
use tracing_log::log::info;

use super::Monzo;
use crate::error::AppErrors as Error;
use crate::model::transaction::{TransactionResponse, TransactionsResponse};

impl Monzo {
    /// Get maximum of [limit] transactions for the given account ID within the given date range
    #[tracing::instrument(name = "Get transactions", skip(self))]
    pub async fn transactions(
        &self,
        account_id: &str,
        since: &NaiveDateTime,
        before: &NaiveDateTime,
        limit: Option<u32>,
    ) -> Result<Vec<TransactionResponse>, Error> {
        let url = format!(
            "{}transactions?account_id={}&since={}&before={}&limit={}&expand[]=merchant",
            self.base_url,
            account_id,
            since.format("%Y-%m-%dT%H:%M:%SZ"),
            before.format("%Y-%m-%dT%H:%M:%SZ"),
            limit.unwrap_or(100)
        );
        info!("url: {}", url);

        let response = self.client.get(&url).send().await?;

        let transactions: TransactionsResponse = Self::handle_response(response).await?;
        let txs_response = transactions.transactions;

        Ok(txs_response)
    }
}

// -- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;

    use crate::{
        model::transaction::TransactionResponse,
        tests::{self, test::get_client},
    };

    use crate::date_ranges;

    #[tokio::test]
    async fn transactions_work() {
        let monzo = get_client();
        let pool = tests::test::test_db().await;

        let mut txs: Vec<TransactionResponse> = Vec::new();
        let account_id = "acc_0000AdNaq81vwtbTBedL06";

        let start =
            NaiveDateTime::parse_from_str("2024-04-01 12:23:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2024-05-21 12:23:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let monthly_intervals = date_ranges(start, end, 30);

        println!("->> {:?}", monthly_intervals.clone());

        for (since, before) in monthly_intervals.clone() {
            let transactions = monzo
                .transactions(account_id, &since, &before, None)
                .await
                .unwrap();

            txs.extend(transactions);
        }

        assert!(txs.len() > 0);
    }
}
