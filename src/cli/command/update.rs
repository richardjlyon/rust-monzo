//! Update transactions
//!
//! This command will fetch transactions from Monzo. By default, it will fetch
//! all transactions since the last. Flag `--all` can be used to reset the
//! database and refetch all transactions.

use std::collections::HashMap;

use rusty_money::{iso, Money};
use tracing_log::log::{error, info};

use crate::{
    client::{transactions::make_date_range, Monzo},
    error::AppErrors as Error,
    model::{
        account::{Account, Service as AccountService, SqliteAccountService},
        transaction::{Service as TransactionService, SqliteTransactionService, Transaction},
        DatabasePool,
    },
};

/// Update transactions
///
/// This function will fetch all transactions from Monzo, print them to the console,
/// and persist them to the database.
///
/// # Errors
/// Will return errors if the transactions cannot be fetched or persisted.
pub async fn update(connection_pool: DatabasePool) -> Result<(), Error> {
    let (transactions, accounts, account_descriptions) = get_sorted_transactions().await?;

    print_transactions(&transactions, &account_descriptions)?;
    persist_transactions(connection_pool, accounts, transactions).await?;

    Ok(())
}

// Get all transactions sorted by date
async fn get_sorted_transactions(
) -> Result<(Vec<Transaction>, Vec<Account>, HashMap<String, String>), Error> {
    let monzo = Monzo::new()?;
    let accounts = monzo.accounts().await?;
    let account_descriptions = monzo.account_description_from_id().await?;
    let (since, before) = make_date_range(2024, 5);
    let mut txs: Vec<Transaction> = Vec::new();

    for account in monzo.accounts().await? {
        let transactions = monzo.transactions(&account.id, since, before, None).await?;

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
fn print_transactions(
    transactions: &Vec<Transaction>,
    account_description: &HashMap<String, String>,
) -> Result<(), Error> {
    println!("{:>85}", "TRANSACTIONS");
    println!(
        "-------------------------------------------------------------------------------------"
    );

    for tx in transactions {
        let account_name = match account_description.get(&tx.account_id) {
            Some(n) => n,
            None => "None",
        };

        let Some(iso_code) = iso::find(&tx.currency) else {
            return Err(Error::CurrencyNotFound(tx.currency.clone()));
        };
        let Some(local_iso_code) = iso::find(&tx.local_currency) else {
            return Err(Error::CurrencyNotFound(tx.local_currency.clone()));
        };

        let date_fmt = tx.created.format("%Y-%m-%d").to_string();
        let amount_fmt = Money::from_minor(tx.amount, iso_code).to_string();

        let credit_fmt = if tx.amount >= 0 {
            amount_fmt.clone()
        } else {
            String::new()
        };

        let debit_fmt = if tx.amount < 0 {
            amount_fmt
        } else {
            String::new()
        };

        let local_amount_fmt = if iso_code == local_iso_code {
            String::new()
        } else {
            format!("({})", Money::from_minor(tx.local_amount, local_iso_code))
        };

        let merchant_fmt = match &tx.merchant {
            Some(merchant) => merchant.name.clone(),
            None => String::new(),
        };

        println!(
            "{:<11} {:<8} {:>12} {:>12} {:>12} {:>25}",
            date_fmt, &account_name, credit_fmt, debit_fmt, local_amount_fmt, merchant_fmt
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
    let account_service = SqliteAccountService::new(connection_pool.clone());
    let tx_service = SqliteTransactionService::new(connection_pool.clone());

    for account in accounts {
        match account_service.create_account(&account).await {
            Ok(()) => info!("Added account: {}", account.id),
            Err(Error::Duplicate(_)) => (),
            Err(e) => {
                error!("Adding account: {}", account.id);
                return Err(e);
            }
        }
    }

    for tx in transactions {
        match tx_service.create_transaction(&tx).await {
            Err(Error::Duplicate(_)) | Ok(()) => (),
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
