//! Update transactions
//!
//! This command will fetch transactions from Monzo. By default, it will fetch
//! all transactions since the last. Flag `--all` can be used to reset the
//! database and refetch all transactions.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use rusty_money::{iso, Money};
use serde::Deserialize;
use tracing_log::log::{error, info};

use crate::{
    client::Monzo,
    date_ranges,
    error::AppErrors as Error,
    model::{
        account::{AccountForDB, Service as AccountService, SqliteAccountService},
        category::{Category, Service as CategoryService, SqliteCategoryService},
        merchant::Merchant,
        pot::{Pot, Service, SqlitePotService},
        transaction::{
            Service as TransactionService, SqliteTransactionService, TransactionResponse,
        },
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
    since: NaiveDateTime,
    before: NaiveDateTime,
) -> Result<(), Error> {
    let (accounts, account_names) = get_accounts(connection_pool.clone()).await?;
    persist_accounts(connection_pool.clone(), &accounts).await?;

    let (pots, pot_names) = get_pots(connection_pool.clone(), &accounts).await?;
    persist_pots(connection_pool.clone(), &pots).await?;

    let txs_resp = get_sorted_transactions(&accounts, since, before).await?;
    persist_categories(connection_pool.clone(), &txs_resp).await?;
    persist_transactions(connection_pool.clone(), &txs_resp).await?;

    print_transactions(&txs_resp, &account_names, &pot_names)?;

    Ok(())
}

// Get all accounts
#[tracing::instrument(name = "get accounts")]
async fn get_accounts(
    connection_pool: DatabasePool,
) -> Result<(Vec<AccountForDB>, HashMap<String, String>), Error> {
    let monzo = Monzo::new()?;
    let accounts = monzo.accounts().await?;
    // convert account response to account for db
    let accounts: Vec<AccountForDB> = accounts.into_iter().map(|account| account.into()).collect();
    let account_names = monzo.account_description_from_id().await?;

    Ok((accounts, account_names))
}

// Get all pots
#[tracing::instrument(name = "get pots")]
async fn get_pots(
    connection_pool: DatabasePool,
    accounts: &Vec<AccountForDB>,
) -> Result<(Vec<Pot>, HashMap<String, String>), Error> {
    let monzo = Monzo::new()?;
    let pot_names = monzo.pot_description_from_id().await?;

    let mut pots: Vec<Pot> = Vec::new();
    for account in accounts {
        let account_pots = monzo.pots(&account.id).await?;
        for pot_resp in account_pots {
            pots.push(Pot::from((pot_resp, account.owner_type.clone())));
        }
    }

    Ok((pots, pot_names))
}

// Get all transactions sorted by date
#[tracing::instrument(name = "get sorted transactions")]
async fn get_sorted_transactions(
    accounts: &Vec<AccountForDB>,
    since: NaiveDateTime,
    before: NaiveDateTime,
) -> Result<Vec<TransactionResponse>, Error> {
    let monzo = Monzo::new()?;
    let mut txs_resp: Vec<TransactionResponse> = Vec::new();

    const DAYS: i64 = 30;

    let date_ranges = date_ranges(since, before, DAYS);

    for account in accounts {
        for (since, before) in date_ranges.clone() {
            let transactions = monzo
                .transactions(&account.id, &since, &before, None)
                .await?;

            info!("Fetched {} transactions", &transactions.len());

            for tx in transactions {
                if tx.amount == 0 || tx.settled.is_none() {
                    continue;
                }

                txs_resp.push(tx);
            }
        }
    }

    // sort by date
    txs_resp.sort_by(|a, b| a.created.cmp(&b.created));

    Ok(txs_resp)
}

/// Print the transactions to the console
fn print_transactions(
    transactions: &Vec<TransactionResponse>,
    account_names: &HashMap<String, String>,
    pot_names: &HashMap<String, String>,
) -> Result<(), Error> {
    println!("{:>85}", "TRANSACTIONS");
    println!(
        "---------------------------------------------------------------------------------------------------------------------"
    );

    for tx in transactions {
        let date_fmt = format_date(&tx.created);

        let account_name_fmt = format_account_name(account_names, &tx.account_id);
        let pot_fmt = format_pot(pot_names, &tx.description);
        let amount = amount_with_currency(tx.amount, &tx.currency)?;
        let credit_fmt = format_credit(tx.amount, &amount);
        let debit_fmt = format_debit(tx.amount, &amount);
        let local_amount_fmt =
            local_amount_with_currency(tx.local_amount, &tx.currency, &tx.local_currency)?;

        let merchant_fmt = format_merchant(&tx.merchant);

        let notes = match &tx.notes {
            Some(d) => d,
            None => "",
        };

        let description_fmt = format_description(notes, &tx.description, pot_names);

        println!(
            "{date_fmt:<11} {account_name_fmt:<8} {pot_fmt:<25} {credit_fmt:>12} {debit_fmt:>12} {local_amount_fmt:>12} {merchant_fmt:>30}  {description_fmt:<30} ",
        );
    }

    Ok(())
}

async fn persist_accounts(
    connection_pool: DatabasePool,
    accounts: &Vec<AccountForDB>,
) -> Result<(), Error> {
    let account_service = SqliteAccountService::new(connection_pool.clone());
    for account in accounts {
        match account_service.save_account(account).await {
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

async fn persist_pots(connection_pool: DatabasePool, pots: &Vec<Pot>) -> Result<(), Error> {
    let pot_service = SqlitePotService::new(connection_pool.clone());
    for pot in pots {
        match pot_service.save_pot(pot).await {
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

async fn persist_categories(
    connection_pool: DatabasePool,
    transactions: &[TransactionResponse],
) -> Result<(), Error> {
    let category_service = SqliteCategoryService::new(connection_pool.clone());

    let categories_config = Categories::from_config()?;
    let custom_categories = categories_config.custom_categories;

    for tx_resp in transactions {
        let category_id = tx_resp.category.clone();
        let category_name = get_category_name(&custom_categories, &category_id);
        let category = Category {
            id: category_id,
            name: category_name,
        };
        match category_service.save_category(&category).await {
            Ok(_) => (),
            Err(Error::Duplicate(_)) => (),
            Err(e) => return Err(Error::DbError(e.to_string())),
        }
    }

    Ok(())
}

// Map a category name from the cateogy_id in the transaction that Monzo uses for custom categories
fn get_category_name(opt_map: &Option<HashMap<String, String>>, key: &str) -> String {
    opt_map
        .as_ref()
        .and_then(|map| map.get(&key.to_lowercase()).cloned())
        .unwrap_or(key.to_string())
}

async fn persist_transactions(
    connection_pool: DatabasePool,
    transactions: &[TransactionResponse],
) -> Result<(), Error> {
    let tx_service = SqliteTransactionService::new(connection_pool.clone());

    for tx_resp in transactions {
        match tx_service.save_transaction(&tx_resp).await {
            Ok(()) => info!("Added transaction: {}", tx_resp.id),
            Err(Error::Duplicate(_)) => (),
            Err(e) => {
                error!("Adding transaction: {}", tx_resp.id);
                return Err(e);
            }
        }
    }

    Ok(())
}

fn amount_with_currency(amount: i64, iso_code: &str) -> Result<String, Error> {
    let Some(iso_code) = iso::find(iso_code) else {
        return Err(Error::CurrencyNotFound(iso_code.to_string()));
    };

    Ok(Money::from_minor(amount, iso_code).to_string())
}

fn local_amount_with_currency(
    amount: i64,
    iso_code: &str,
    local_iso_code: &str,
) -> Result<String, Error> {
    if iso_code == local_iso_code {
        return Ok(String::new());
    }

    let Some(iso_code) = iso::find(local_iso_code) else {
        return Err(Error::CurrencyNotFound(iso_code.to_string()));
    };

    Ok(format!("({})", Money::from_minor(amount, iso_code)))
}

fn format_date(date: &DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn format_account_name(account_names: &HashMap<String, String>, account_id: &str) -> String {
    match account_names.get(account_id) {
        Some(description) => description.clone(),
        None => account_id.to_string(),
    }
}

fn format_pot(pot_names: &HashMap<String, String>, description: &str) -> String {
    let pot_fmt = match pot_names.get(description) {
        Some(description) => description.clone(),
        None => String::new(),
    };

    pot_fmt
}

fn format_credit(amount: i64, amount_str: &str) -> String {
    if amount >= 0 {
        amount_str.to_string()
    } else {
        String::new()
    }
}

fn format_debit(amount: i64, amount_str: &str) -> String {
    if amount < 0 {
        amount_str.to_string()
    } else {
        String::new()
    }
}

fn format_merchant(merchant: &Option<Merchant>) -> String {
    match merchant {
        Some(merchant) => merchant.name.clone(),
        None => String::new(),
    }
}

fn format_description(
    notes: &str,
    description: &str,
    pot_names: &HashMap<String, String>,
) -> String {
    // substitute the description with the pot name if it exists
    let description_with_pot_name = match pot_names.get(description) {
        Some(pot_name) => format!("Pot:{}", pot_name.clone()),
        None => description.to_string(),
    };

    let description_fmt = match notes.len() {
        0 => description_with_pot_name,
        _ => notes.to_string(),
    };

    description_fmt.to_string()
}

#[derive(Debug, Deserialize)]
struct Categories {
    custom_categories: Option<HashMap<String, String>>,
}

impl Categories {
    pub fn from_config() -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::File::new(
                "categories.yaml",
                config::FileFormat::Yaml,
            ))
            .build()?;

        match cfg.try_deserialize::<Categories>() {
            Ok(custom_categories) => Ok(custom_categories),
            Err(e) => {
                println!("{}", e.to_string());
                Err(Error::ConfigurationError(e))
            }
        }
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount() {
        let mut res = amount_with_currency(10000, "GBP").unwrap();
        assert_eq!(res, "£100.00");

        res = amount_with_currency(10000, "USD").unwrap();
        assert_eq!(res, "$100.00");
    }

    #[test]
    fn test_amount_error() {
        let res = amount_with_currency(10000, "XXX");
        assert!(res.is_err());
    }

    #[test]
    fn test_local_amount() {
        let res = local_amount_with_currency(10000, "GBP", "GBP").unwrap();
        assert_eq!(res, "");

        let res = local_amount_with_currency(10000, "GBP", "USD").unwrap();
        assert_eq!(res, "($100.00)");
    }

    #[test]
    fn test_local_amount_error() {
        let res = local_amount_with_currency(10000, "USD", "XXX");
        assert!(res.is_err());
    }
}
