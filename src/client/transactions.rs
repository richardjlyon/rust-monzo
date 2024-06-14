//! Transaction related functions
//!
//! This module gets transaction information from the Monzo API.

use chrono::{DateTime, Utc};
use tracing_log::log::info;

use super::Monzo;
use crate::error::AppErrors as Error;
use crate::model::transaction::{TransactionResponse, Transactions};

impl Monzo {
    /// Get maximum of [limit] transactions for the given account ID within the given date range
    #[tracing::instrument(name = "Get transactions", skip(self))]
    pub async fn transactions(
        &self,
        account_id: &str,
        since: &DateTime<Utc>,
        before: &DateTime<Utc>,
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

        let transactions: Transactions = Self::handle_response(response).await?;
        let txs_response = transactions.transactions;

        Ok(txs_response)
    }
}

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use chrono_intervals::{Grouping, IntervalGenerator};

    use crate::{
        model::transaction::TransactionResponse,
        tests::{self, test::get_client},
    };

    #[tokio::test]
    async fn transactions_work() {
        let monzo = get_client();
        let pool = tests::test::test_db().await;

        let mut txs: Vec<TransactionResponse> = Vec::new();
        let account_id = "acc_0000AdNaq81vwtbTBedL06";

        let since = DateTime::parse_from_rfc3339("2024-04-01T12:23:45.000000-07:00").unwrap();
        let before = DateTime::parse_from_rfc3339("2024-05-21T12:23:45.000000-07:00").unwrap();
        let monthly_intervals = IntervalGenerator::new()
            .with_grouping(Grouping::PerMonth)
            .get_intervals(since, before);

        println!("->> {:?}", monthly_intervals.clone());

        for (since, before) in monthly_intervals.clone() {
            println!("->> {} - {}", since, before);
            let transactions = monzo
                .transactions(account_id, &since, &before, None)
                .await
                .unwrap();

            txs.extend(transactions);
        }

        assert!(txs.len() > 0);
    }
}
