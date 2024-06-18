//! Beancount

use std::{fs::File, io::Write};

use chrono::{Local, NaiveTime};
use config::Case;
use convert_case::Casing;
use rusty_money::{iso, Money};

use crate::{
    beancount::{Account, AccountType, Beancount, Directive, Posting, Postings, Transaction},
    date_ranges,
    error::AppErrors as Error,
    model::{
        account::{Service as AccountService, SqliteAccountService},
        category::Category,
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

    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let config = Beancount::from_config()?;
    let since = config.settings.start_date.and_time(time);
    let before = Local::now().naive_local();
    let date_ranges = date_ranges(since, before, 30);

    let service = SqliteTransactionService::new(pool.clone());

    // -- Open Equity Accounts -----------------------------------------------------

    directives.push(Directive::Comment("equity accounts".to_string()));
    directives.extend(open_config_equity_assets()?);

    // -- Open Asset Accounts --------------------------------------------------------------

    directives.push(Directive::Comment("asset accounts".to_string()));
    directives.extend(open_monzo_assets(pool.clone()).await?);
    directives.extend(open_config_assets()?);
    directives.extend(open_monzo_pot_liabilities(pool.clone()).await?);

    // -- Open Expense Accounts  ---------------------------------------------------------

    directives.push(Directive::Comment("expense accounts".to_string()));
    directives.extend(open_monzo_expenses(pool.clone()).await?);

    // Open Liability Accounts ---------------------------------------------------------

    // directives.push(Directive::Comment("liabilities".to_string()));
    // directives.extend(open_config_liabilities().await?);

    // -- Post Transactions-------------------------------------------------------------

    directives.push(Directive::Comment("transactions".to_string()));

    for (since, before) in date_ranges {
        let transactions = service.read_beancount_data(since, before).await?;

        for tx in transactions {
            let to_posting = prepare_to_posting(&pool, &tx).await?;
            let from_posting = prepare_from_posting(&tx);

            let postings = Postings {
                to: to_posting,
                from: from_posting,
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

fn open_config_equity_assets() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(equity_accounts) = bc.settings.equities {
        for equity in equity_accounts {
            directives.push(Directive::Open(bc.settings.start_date, equity, None));
        }
    }

    Ok(directives)
}

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
            account_name: account.owner_type,
            label: None,
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
                account_name: pot.name,
                label: None,
            };
            directives.push(Directive::Open(open_date, beanaccount, None));
        }
        None => (),
    }

    Ok(directives)
}

fn open_config_assets() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    match bc.settings.assets {
        Some(asset_accounts) => {
            for asset_account in asset_accounts {
                directives.push(Directive::Open(open_date, asset_account, None));
            }
        }
        None => (),
    }

    Ok(directives)
}

// Open an expense account for each category in each account
//
// We filter out category names corresponding to Assets, as defined in the config
async fn open_monzo_expenses(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;

    let account_service = SqliteAccountService::new(pool.clone());
    let transaction_service = SqliteTransactionService::new(pool.clone());

    let mut directives: Vec<Directive> = Vec::new();

    // remove categories for which a config asset account of the
    // same account name and category name exists

    let asset_accounts = bc.settings.assets.unwrap();
    let accounts = account_service.read_accounts().await?;

    for account in accounts {
        let account_categories = transaction_service
            .get_categories_for_account(&account.id)
            .await?;

        let valid_categories: Vec<Category> = account_categories
            .iter()
            .filter(|c| {
                !asset_accounts.iter().any(|a| {
                    a.label.as_deref().unwrap_or_default().to_case(Case::Lower)
                        == c.name.to_case(Case::Lower)
                })
            })
            .cloned()
            .collect();

        for category in valid_categories {
            let beanaccount = Account {
                account_type: AccountType::Expenses,
                currency: account.currency.clone(),
                account_name: account.owner_type.clone(),
                label: Some(category.name),
            };
            directives.push(Directive::Open(open_date, beanaccount, None));
        }
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
                    account_name: account.owner_type.clone().to_case(Case::Pascal),
                    label: Some(pot.name.to_case(Case::Pascal)),
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
            account_name: bc.settings.provider.clone(),
            label: account.label,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

// Prepare a beancount liability posting
//
// We have to handle three edge cases:
// 1. transfers into accounts
// 2. transfers between accounts and pots that aren't the flexible_savings` pot, and
// 3. transfers between accounts and the `flexible_savings` pot.
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
async fn prepare_to_posting(
    pool: &DatabasePool,
    tx: &BeancountTransaction,
) -> Result<Posting, Error> {
    let mut account = Account {
        account_type: AccountType::Liabilities,
        currency: tx.currency.clone(),
        account_name: tx.account_name.clone().to_case(Case::Pascal),
        label: Some(tx.category_name.clone().to_case(Case::Pascal)),
    };

    let mut amount = -tx.amount as f64;

    match tx.category_name.as_str() {
        "transfers" => {
            if tx.description.starts_with("Monzo-") {
                account.account_type = AccountType::Assets;
                account.label = Some("income".to_string());
                amount = tx.amount as f64;
            } else if tx.category_name == "savings" {
                account.account_type = AccountType::Assets;
                account.label = Some("savings".to_string());
            } else {
                let pot_service = SqlitePotService::new(pool.clone());
                account.label = match pot_service.read_pot_by_id(&tx.description).await? {
                    Some(pot) => Some(pot.name),
                    None => Some(tx.description.clone()),
                };
            }
        }
        _ => {}
    }

    Ok(Posting {
        account,
        amount,
        currency: tx.currency.to_string(),
        description: Some(tx.description.clone()),
    })
}

// FIXME: Handle transferring from the flexible_savings pot
fn prepare_from_posting(tx: &BeancountTransaction) -> Posting {
    let account = Account {
        account_type: AccountType::Assets,
        currency: tx.currency.to_string(),
        account_name: tx.account_name.to_string(),
        label: None,
    };

    Posting {
        account,
        amount: tx.amount as f64,
        currency: tx.currency.clone(),
        description: None,
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
