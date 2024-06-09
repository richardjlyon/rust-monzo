use std::collections::HashMap;

use rusty_money::{iso, Money};

use crate::{
    client::{transactions::make_date_range, MonzoClient},
    error::AppError as Error,
    model::{
        account::{Account, AccountService, SqliteAccountService},
        transaction::Transaction,
        DatabasePool,
    },
};

/// Update transactions
///
/// This function will fetch all transactions from Monzo, print them to the console,
/// and persist them to the database.
pub async fn update(connection_pool: DatabasePool) -> Result<(), Error> {
    let (transactions, accounts, account_descriptions) = get_sorted_transactions().await?;

    print_transactions(&transactions, &account_descriptions).await?;
    persist_transactions(connection_pool, accounts, transactions).await?;

    Ok(())
}

// Get all transactions sorted by date
async fn get_sorted_transactions(
) -> Result<(Vec<Transaction>, Vec<Account>, HashMap<String, String>), Error> {
    let monzo = MonzoClient::new()?;
    let accounts = monzo.accounts().await?;
    let account_descriptions = monzo.account_description_from_id().await?;
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

    Ok((txs, accounts, account_descriptions))
}

/// Print the transactions to the console
async fn print_transactions(
    transactions: &Vec<Transaction>,
    account_description: &HashMap<String, String>,
) -> Result<(), Error> {
    println!("{:>85}", "TRANSACTIONS");
    println!(
        "-------------------------------------------------------------------------------------"
    );

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

        let merchant_fmt = match &tx.merchant {
            Some(merchant) => merchant.name.to_owned(),
            None => "".to_string(),
        };

        println!(
            "{:<11} {:<8} {:>12} {:>12} {:>12} {:>25}",
            date_fmt, account_name, credit_fmt, debit_fmt, local_amount_fmt, merchant_fmt
        );
    }

    Ok(())
}

// Persist the transactions to the database
async fn persist_transactions(
    connection_pool: DatabasePool,
    accounts: Vec<Account>,
    transactions: Vec<Transaction>,
) -> Result<(), Error> {
    let service = SqliteAccountService::new(connection_pool);

    for account in accounts {
        service.create_account(&account).await?;
    }

    Ok(())
}
