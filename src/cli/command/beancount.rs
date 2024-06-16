//! Beancount

use chrono::NaiveDateTime;
use rusty_money::{iso, Money};

use crate::{
    beancount::{
        AccountType, AssetAccount, AssetPosting, Beancount, Directive, LiabilityAccount,
        LiabilityPosting, Postings, Transaction,
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

pub async fn beancount(pool: DatabasePool) -> Result<(), Error> {
    let mut directives: Vec<Directive> = Vec::new();

    // Open assets
    directives.push(Directive::Comment("assets".to_string()));
    directives.extend(monzo_assets(pool.clone()).await?);
    directives.extend(monzo_pots(pool.clone()).await?);
    directives.extend(config_assets()?);

    // Open liabilities
    directives.push(Directive::Comment("liabilities".to_string()));
    directives.extend(config_liabilities()?);

    // Open equity accounts
    directives.push(Directive::Comment("equities".to_string()));
    directives.extend(config_equities()?);

    // Banking - Get January `Personal` transactions
    directives.push(Directive::Comment("transactions".to_string()));

    let service = SqliteTransactionService::new(pool);
    let start = NaiveDateTime::parse_from_str("2024-04-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let end = NaiveDateTime::parse_from_str("2024-04-30 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();
    let date_ranges = date_ranges(start, end, 30);

    // First pass: Get all transactions
    for (since, before) in date_ranges {
        let transactions = service.read_beancount_data(since, before).await?;

        for tx in transactions {
            let liability_posting = prepare_liability_posting(&tx);
            let asset_posting = prepare_asset_posting(&tx);

            let postings = Postings {
                liability_posting,
                asset_posting,
            };

            let transaction = prepare_transaction(&tx, &postings);

            directives.push(Directive::Transaction(transaction));
        }
    }

    // Second pass: Process transactions to combine transfers
    // TODO: Combine transfers

    for d in directives {
        println!("{}", d.to_formatted_string());
    }

    Ok(())
}

fn prepare_liability_posting(tx: &BeancountTransaction) -> LiabilityPosting {
    let liability_account = LiabilityAccount {
        account_type: AccountType::Liabilities,
        currency: tx.local_currency.clone(),
        category: tx.category.clone(),
    };

    LiabilityPosting {
        account: liability_account,
        amount: -tx.amount as f64,
        currency: tx.currency.to_string(),
        description: String::new(),
    }
}

fn prepare_asset_posting(tx: &BeancountTransaction) -> AssetPosting {
    let asset_account = AssetAccount {
        account_type: AccountType::Assets,
        currency: tx.currency.to_string(),
        provider: "Monzo".to_string(),
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

async fn monzo_assets(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let acc_service = SqliteAccountService::new(pool.clone());
    let mut directives: Vec<Directive> = Vec::new();
    let accounts = acc_service.read_accounts().await?;

    for account in accounts {
        let beanaccount = AssetAccount {
            account_type: AccountType::Assets,
            currency: account.currency,
            provider: "Monzo".to_string(),
            name: account.owner_type,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

async fn monzo_pots(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let pot_service = SqlitePotService::new(pool.clone());
    let mut directives: Vec<Directive> = Vec::new();
    let pots = pot_service.read_pots().await?;
    for pot in pots {
        let beanaccount = AssetAccount {
            account_type: AccountType::Assets,
            currency: pot.currency,
            provider: "Monzo".to_string(),
            name: pot.name,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

fn config_assets() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.settings.assets.is_none() {
        return Ok(directives);
    }

    for account in bc.settings.assets.unwrap() {
        let beanaccount = AssetAccount {
            name: account.name,
            account_type: AccountType::Assets,
            currency: account.currency,
            provider: account.provider,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

fn config_liabilities() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.settings.liabilities.is_none() {
        return Ok(directives);
    }

    for account in bc.settings.liabilities.unwrap() {
        let beanaccount = AssetAccount {
            name: account.name,
            account_type: AccountType::Liabilities,
            currency: account.currency,
            provider: account.provider,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}

fn config_equities() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.settings.equities.is_none() {
        return Ok(directives);
    }

    for account in bc.settings.equities.unwrap() {
        let beanaccount = AssetAccount {
            name: account.name,
            account_type: AccountType::Equities,
            currency: account.currency,
            provider: account.provider,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}
