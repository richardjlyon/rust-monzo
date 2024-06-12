//! Update transactions
//!
//! This command will fetch transactions from Monzo. By default, it will fetch
//! all transactions since the last. Flag `--all` can be used to reset the
//! database and refetch all transactions.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use chrono_intervals::{Grouping, IntervalGenerator};
use rusty_money::{iso, Money};
use tracing_log::log::{error, info};

use crate::{
    client::Monzo,
    error::AppErrors as Error,
    model::{
        account::{Account, Service as AccountService, SqliteAccountService},
        pots::{Pot, Service, SqlitePotService},
        transaction::{Service as TransactionService, SqliteTransactionService, Transaction},
        DatabasePool,
    },
};

/// Update transactions
///
/// This function will fetch transactions from Monzo between the given dates,
/// print them to the console, and persist them to the database.
///
/// # Errors
/// Will return errors if the transactions cannot be fetched or persisted.
pub async fn update(
    connection_pool: DatabasePool,
    since: &DateTime<Utc>,
    before: &DateTime<Utc>,
) -> Result<(), Error> {
    let (accounts, account_descriptions) = get_accounts(connection_pool.clone()).await?;
    persist_accounts(connection_pool.clone(), &accounts).await?;

    let pots = get_pots(connection_pool.clone(), &accounts).await?;
    persist_pots(connection_pool.clone(), &pots).await?;

    let transactions = get_sorted_transactions(&accounts, &since, before).await?;
    info!("->> Fetched {} transactions", transactions.len());
    persist_transactions(connection_pool.clone(), &transactions).await?;

    print_transactions(&transactions, &account_descriptions)?;

    Ok(())
}

// Get all accounts
#[tracing::instrument(name = "get accounts")]
async fn get_accounts(
    connection_pool: DatabasePool,
) -> Result<(Vec<Account>, HashMap<String, String>), Error> {
    let monzo = Monzo::new()?;
    let accounts = monzo.accounts().await?;
    let account_descriptions = monzo.account_description_from_id().await?;

    Ok((accounts, account_descriptions))
}

// Get all pots
#[tracing::instrument(name = "get pots")]
async fn get_pots(
    connection_pool: DatabasePool,
    accounts: &Vec<Account>,
) -> Result<Vec<Pot>, Error> {
    let monzo = Monzo::new()?;
    let mut pots: Vec<Pot> = Vec::new();

    for account in accounts {
        let account_pots = monzo.pots(&account.id).await?;
        pots.extend(account_pots);
    }

    Ok(pots)
}

// Get all transactions sorted by date
#[tracing::instrument(name = "get sorted transactions")]
async fn get_sorted_transactions(
    accounts: &Vec<Account>,
    since: &DateTime<Utc>,
    before: &DateTime<Utc>,
) -> Result<Vec<Transaction>, Error> {
    let monzo = Monzo::new()?;

    let mut txs: Vec<Transaction> = Vec::new();

    let monthly_intervals = IntervalGenerator::new()
        .with_grouping(Grouping::PerMonth)
        .get_intervals(*since, *before);

    for account in accounts {
        for (since, before) in monthly_intervals.clone() {
            let transactions = monzo
                .transactions(&account.id, &since, &before, None)
                .await?;

            info!("Fetched {} transactions", &transactions.len());

            for tx in transactions {
                if tx.amount == 0 || tx.settled.is_none() {
                    continue;
                }

                txs.push(tx);
            }
        }
    }

    // sort by date
    txs.sort_by(|a, b| a.created.cmp(&b.created));

    info!("END");

    Ok(txs)
}

/// Print the transactions to the console
fn print_transactions(
    transactions: &Vec<Transaction>,
    account_description: &HashMap<String, String>,
) -> Result<(), Error> {
    println!("{:>85}", "TRANSACTIONS");
    println!(
        "---------------------------------------------------------------------------------------------------------------------"
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

        let description = match tx.notes.len() {
            0 => tx.description.clone(),
            _ => tx.notes.clone(),
        };

        println!(
            "{:<11} {:<8} {:>12} {:>12} {:>12} {:>30}  {:<30} ",
            date_fmt,
            &account_name,
            credit_fmt,
            debit_fmt,
            local_amount_fmt,
            merchant_fmt,
            description
        );
    }

    Ok(())
}

// Persist the accounts to the database
async fn persist_accounts(
    connection_pool: DatabasePool,
    accounts: &Vec<Account>,
) -> Result<(), Error> {
    let account_service = SqliteAccountService::new(connection_pool.clone());
    for account in accounts {
        match account_service.save_account(&account).await {
            Ok(()) => info!("Added account: {}", account.id),
            Err(Error::Duplicate(_)) => (),
            Err(e) => {
                error!("Adding account: {}", account.id);
                return Err(e);
            }
        }
    }

    Ok(())
}

// Persist the pots to the database
async fn persist_pots(connection_pool: DatabasePool, pots: &Vec<Pot>) -> Result<(), Error> {
    let pot_service = SqlitePotService::new(connection_pool.clone());
    for pot in pots {
        match pot_service.save_pot(&pot).await {
            Ok(()) => info!("Added pot: {}", pot.id),
            Err(Error::Duplicate(_)) => (),
            Err(e) => {
                error!("Adding pot: {}", pot.id);
                return Err(e);
            }
        }
    }

    Ok(())
}

// Persist the transactions to the database
async fn persist_transactions(
    connection_pool: DatabasePool,
    transactions: &Vec<Transaction>,
) -> Result<(), Error> {
    let tx_service = SqliteTransactionService::new(connection_pool.clone());

    for tx in transactions {
        match tx_service.save_transaction(&tx).await {
            Ok(()) => info!("Added transaction: {}", tx.id),
            Err(Error::Duplicate(_)) => (),
            Err(e) => {
                error!("Adding transaction: {}", tx.id);
                return Err(e);
            }
        }
    }

    Ok(())
}
