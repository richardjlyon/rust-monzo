use anyhow::Error;
use rusty_money::{iso, Money};

use crate::client::{
    transactions::{make_date_range, Transaction},
    MonzoClient,
};

pub async fn update() -> Result<(), Error> {
    let monzo = MonzoClient::new()?;

    println!("{:>77}", "TRANSACTIONS");
    println!("-----------------------------------------------------------------------------");

    let (since, before) = make_date_range(2024, 5); // TODO: use command parameters

    let _txs: Vec<Transaction> = Vec::new();

    for account in monzo.accounts().await? {
        let transactions = monzo
            .transactions(&account.id, since, before, None)
            .await
            .unwrap();

        for tx in transactions {
            if tx.amount == 0 {
                continue;
            }

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

            let local_amount_fmt = match (iso_code == local_iso_code) {
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
                "{:<12} {:>12} {:>12} {:>12} {:>25}",
                date_fmt, credit_fmt, debit_fmt, local_amount_fmt, merchant_fmt
            );
        }
    }

    Ok(())
}
