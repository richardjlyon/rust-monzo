//! Beancount

use std::{fs::File, io::Write};

use chrono::{Local, NaiveTime};
use config::Case;
use convert_case::Casing;
use rusty_money::{iso, Money};

use crate::{
    beancount::{
        Account, AccountType, AssetPosting, Beancount, Directive, LiabilityPosting, Postings,
        Transaction,
    },
    date_ranges,
    error::AppErrors as Error,
    model::{
        account::{Service as AccountService, SqliteAccountService},
        pot::{Service as PotService, SqlitePotService},
        transaction::{BeancountTransaction, Service, SqliteTransactionService},
        DatabasePool,
    },
};

/// Generates a beancount file from a combination of database entries and entries from
/// the configuration file.
///
/// Implementation notes:
///
/// We treat a Monzo `Pot` as a liability account and a Monzo `Account` as an asset account.
/// However, as an edge case, we need to handle the `Savings` pot as an asset account. This is
/// done with logic in the `prepare_liability_posting` and `prepare_asset_posting` functions to
/// identify the savings pot and set `AccountType`.

pub async fn beancount(pool: DatabasePool) -> Result<(), Error> {
    let mut directives: Vec<Directive> = Vec::new();
    let service = SqliteTransactionService::new(pool.clone());
    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let config = Beancount::from_config()?;

    // Open assets
    directives.push(Directive::Comment("assets".to_string()));
    directives.extend(open_monzo_assets(pool.clone()).await?);
    directives.extend(open_config_assets()?);

    // Open liabilities
    directives.push(Directive::Comment("liabilities".to_string()));
    directives.extend(open_monzo_liabilities(pool.clone()).await?);
    directives.extend(open_monzo_pot_liabilities(pool.clone()).await?);
    directives.extend(open_config_liabilities().await?);

    // Banking - Get January `Personal` transactions
    directives.push(Directive::Comment("transactions".to_string()));

    let since = config.settings.start_date.and_time(time);
    let before = Local::now().naive_local();

    let date_ranges = date_ranges(since, before, 30);

    // First pass: Get all transactions
    for (since, before) in date_ranges {
        let transactions = service.read_beancount_data(since, before).await?;

        for tx in transactions {
            let liability_posting = prepare_liability_posting(&pool, &tx).await?;
            let asset_posting = prepare_asset_posting(&tx);

            let postings = Postings {
                liability_posting,
                asset_posting,
            };

            let transaction = prepare_transaction(&tx, &postings);

            directives.push(Directive::Transaction(transaction));
        }
    }

    let mut file = File::create("report.beancount")?;
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}

// Open assets for Monzo database entities
async fn open_monzo_assets(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let acc_service = SqliteAccountService::new(pool.clone());
    let pot_service = SqlitePotService::new(pool.clone());
    let mut directives: Vec<Directive> = Vec::new();
    let accounts = acc_service.read_accounts().await?;

    // Add the Monzo accounts (i.e."personal", "business") as assets
    for account in accounts {
        let beanaccount = Account {
            account_type: AccountType::Assets,
            currency: account.currency,
            account_name: None,
            name: account.owner_type,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    // Add the Flexible Savings Pot as an asset
    // This is a special case as it is not a transfer
    match pot_service.read_pot_by_type("flexible_savings").await? {
        Some(pot) => {
            let beanaccount = Account {
                account_type: AccountType::Assets,
                currency: pot.currency,
                account_name: None,
                name: pot.name,
            };
            directives.push(Directive::Open(open_date, beanaccount, None));
        }
        None => (),
    }

    Ok(directives)
}

// Ppen assets for configuration file entitites
fn open_config_assets() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.settings.assets.is_none() {
        return Ok(directives);
    }

    for account in bc.settings.assets.unwrap() {
        let beanaccount = Account {
            account_type: AccountType::Assets,
            currency: account.currency,
            account_name: None,
            name: account.name,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

// Open a liability account for each category in each account
//
// An edge case is the "savings" `category_id` which marks transfers to the
// `flexible_savings` Pot and must be excluded.
async fn open_monzo_liabilities(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;

    let account_service = SqliteAccountService::new(pool.clone());
    let transaction_service = SqliteTransactionService::new(pool.clone());

    let mut directives: Vec<Directive> = Vec::new();

    for account in account_service.read_accounts().await? {
        let categories = transaction_service
            .get_categories_for_account(&account.id)
            .await?;

        let valid_categories = categories
            .iter()
            .filter(|c| c.name != "savings")
            .map(|category| {
                let beanaccount = Account {
                    account_type: AccountType::Liabilities,
                    currency: account.currency.clone(),
                    account_name: Some(account.owner_type.to_case(Case::Pascal).clone()),
                    name: category.name.clone(),
                };
                Directive::Open(open_date, beanaccount, None)
            });

        directives.extend(valid_categories);
    }

    Ok(directives)
}

// Open a liability account for each pot
//
// An edge case is the `savings` pot which is given a category of `savings` rather than
// `transfers`. We handle this by checking for the `savings` category and excluding it from the liability as it is
// created  in `monzo_zssets()` from its `category_id`.
async fn open_monzo_pot_liabilities(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;

    let mut directives: Vec<Directive> = Vec::new();

    let account_service = SqliteAccountService::new(pool.clone());
    let transaction_service = SqliteTransactionService::new(pool.clone());

    let accounts = account_service.read_accounts().await?;

    for account in accounts {
        let pots = transaction_service
            .get_pots_for_account(&account.owner_type)
            .await?;

        let valid_pots = pots
            .into_iter()
            .filter(|pot| pot.pot_type != "flexible_savings")
            .map(|pot| {
                let beanaccount = Account {
                    account_type: AccountType::Liabilities,
                    currency: pot.currency,
                    account_name: Some(account.owner_type.clone().to_case(Case::Pascal)),
                    name: pot.name,
                };
                Directive::Open(open_date, beanaccount, None)
            });

        directives.extend(valid_pots);
    }

    Ok(directives)
}

// Open a liability account for each config file entity
async fn open_config_liabilities() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.settings.liabilities.is_none() {
        return Ok(directives);
    }

    // open configured liabilities
    for account in bc.settings.liabilities.unwrap() {
        let beanaccount = Account {
            account_type: AccountType::Liabilities,
            currency: account.currency,
            account_name: Some(bc.settings.provider.clone()),
            name: account.name,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

// Prepare a beancount liability posting
//
// We have to handle three edge cases: transfers into accouts, transfers between accounts and pots that
// aren't the flexible_savings` pot, and transfers between accounts and the `flexible_savings` pot.
//
// Transfers into accounts are assumed to have a description starting with `Monzo-` and are mapped to
// the `income` category.
//
// Transfers between accounts and pots are assumed to have a description that is the pot id and are
// mapped to the pot name.
//
// Transfers between accounts and the `flexible_savings` pot are assumed to have the category `savings`
// and are excluded.
//
async fn prepare_liability_posting(
    pool: &DatabasePool,
    tx: &BeancountTransaction,
) -> Result<LiabilityPosting, Error> {
    let category = map_category_name(pool, &tx.category_name, &tx.description).await?;

    let liability_account = Account {
        account_type: AccountType::Liabilities,
        currency: tx.currency.clone(),
        account_name: Some(tx.account_name.to_case(Case::Pascal).clone()),
        name: category,
    };

    Ok(LiabilityPosting {
        account: liability_account,
        amount: -tx.amount as f64,
        currency: tx.currency.to_string(),
        description: String::new(),
    })
}

fn prepare_asset_posting(tx: &BeancountTransaction) -> AssetPosting {
    let asset_account = Account {
        account_type: AccountType::Assets,
        currency: tx.currency.to_string(),
        account_name: None,
        name: tx.account_name.to_string(),
    };

    AssetPosting {
        account: asset_account,
        amount: tx.amount as f64,
        currency: tx.currency.clone(),
    }
}

fn prepare_transaction(tx: &BeancountTransaction, postings: &Postings) -> Transaction {
    let comment = prepare_transaction_comment(tx);
    let date = tx.settled.unwrap_or(tx.created).date();
    let notes = prepare_transaction_notes(tx);

    Transaction {
        comment,
        date,
        notes,
        postings: postings.clone(),
    }
}

// Map the category name to a beancount liability account name
//
async fn map_category_name(
    pool: &DatabasePool,
    category_name: &str,
    tx_description: &str,
) -> Result<String, Error> {
    if category_name != "transfers" {
        return Ok(category_name.to_string());
    }

    if tx_description.starts_with("Monzo-") {
        return Ok("income".to_string());
    }

    let pot_service = SqlitePotService::new(pool.clone());
    match pot_service.read_pot_by_id(tx_description).await? {
        Some(p) => return Ok(p.name),
        None => return Ok(tx_description.to_string()),
    }
}

fn prepare_transaction_comment(tx: &BeancountTransaction) -> Option<String> {
    let amount = prepare_amount(tx);
    let notes = tx.notes.clone().unwrap();

    Some(format!("{notes} {amount}"))
}

fn prepare_transaction_notes(tx: &BeancountTransaction) -> String {
    let merchant_name = tx.merchant_name.clone().unwrap_or(String::new());

    format!("{}", merchant_name)
}

fn prepare_amount(tx: &BeancountTransaction) -> String {
    if tx.currency == tx.local_currency {
        String::new()
    } else {
        if let Some(iso_code) = iso::find(&tx.local_currency) {
            format!("{}", Money::from_minor(tx.local_amount, iso_code))
        } else {
            format!("{} {}", tx.local_amount, tx.local_currency)
        }
    }
}
