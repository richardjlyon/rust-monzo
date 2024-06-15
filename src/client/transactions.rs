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

#[cfg(test)]
mod test {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use chrono_intervals::{Grouping, IntervalGenerator};

    use crate::{
        model::transaction::TransactionResponse,
        tests::{self, test::get_client},
    };

    #[tokio::test]
    #[ignore = "Need to fix datetime handling"]
    async fn transactions_work() {
        let monzo = get_client();
        let pool = tests::test::test_db().await;

        let mut txs: Vec<TransactionResponse> = Vec::new();
        let account_id = "acc_0000AdNaq81vwtbTBedL06";

        let format = "%Y-%m-%d %H:%M:%S";
        let since_str = "2024-04-01 12:23:-00";
        let before_str = "2024-05-21 12:23:00";

        // TODO: reimplement this mess
        let since = NaiveDateTime::parse_from_str(since_str, format)
            .expect("Failed to parse date and time");
        let before = NaiveDateTime::parse_from_str(before_str, format)
            .expect("Failed to parse date and time");

        let since_utc: DateTime<Utc> = DateTime::from_utc(since, Utc);
        let before_utc: DateTime<Utc> = DateTime::from_utc(before, Utc);

        let monthly_intervals = IntervalGenerator::new()
            .with_grouping(Grouping::PerMonth)
            .get_intervals(since_utc, before_utc);

        println!("->> {:?}", monthly_intervals.clone());

        for (since, before) in monthly_intervals.clone() {
            println!("->> {} - {}", since, before);
            let transactions = monzo
                .transactions(account_id, &since.naive_utc(), &before.naive_utc(), None)
                .await
                .unwrap();

            txs.extend(transactions);
        }

        assert!(txs.len() > 0);
    }
}
