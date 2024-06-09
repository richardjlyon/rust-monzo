use anyhow::Error;
use rusty_money::{iso, Money};

use crate::{
    client::{transactions::make_date_range, MonzoClient},
    model::transaction::Transaction,
};

pub async fn update() -> Result<(), Error> {
    let monzo = MonzoClient::new()?;

    println!("{:>85}", "TRANSACTIONS");
    println!(
        "-------------------------------------------------------------------------------------"
    );

    let transactions = get_sorted_transactions().await?;
    let account_description = monzo.account_description_from_id().await?;

    for tx in transactions {
        let account_name = account_description.get(&tx.account_id).unwrap();

        let iso_code = iso::find(&tx.currency).unwrap();
        let local_iso_code = iso::find(&tx.local_currency).unwrap();

        let date_fmt = tx.created.format("%Y-%m-%d").to_string();
        let amount_fmt = Money::from_minor(tx.amount, iso_code).to_string();

        let credit_fmt = match tx.amount >= 0 {
            true => amount_fmt.clone(),
            false => "".to_string(),
        };

        let debit_fmt = match tx.amount < 0 {
            true => amount_fmt,
            false => "".to_string(),
        };

        let local_amount_fmt = match iso_code == local_iso_code {
            true => "".to_string(),
            false => format!(
                "({})",
                Money::from_minor(tx.local_amount, local_iso_code).to_string()
            ),
        };

        let merchant_fmt = match tx.merchant {
            Some(merchant) => merchant.name,
            None => "".to_string(),
        };

        println!(
            "{:<11} {:<8} {:>12} {:>12} {:>12} {:>25}",
            date_fmt, account_name, credit_fmt, debit_fmt, local_amount_fmt, merchant_fmt
        );
    }

    Ok(())
}

/// Get all transactions sorted by date
async fn get_sorted_transactions() -> Result<Vec<Transaction>, Error> {
    let monzo = MonzoClient::new()?;
    let (since, before) = make_date_range(2024, 5);
    let mut txs: Vec<Transaction> = Vec::new();

    for account in monzo.accounts().await? {
        let transactions = monzo
            .transactions(&account.id, since, before, None)
            .await
            .unwrap();

        for tx in transactions {
            if tx.amount == 0 {
                continue;
            }

            txs.push(tx);
        }
    }

    // sort by date
    txs.sort_by(|a, b| a.created.cmp(&b.created));

    Ok(txs)
}
