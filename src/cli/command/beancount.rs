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
        transaction::{Service, SqliteTransactionService},
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

    for (since, before) in date_ranges {
        let beancount_data = service.read_beancount_data(since, before).await?;
        println!("->> {:#?}", beancount_data);
        for tx in beancount_data {
            let to_description = "".to_string();

            let liability_account = LiabilityAccount {
                account_type: AccountType::Liabilities,
                currency: tx.local_currency.clone(),
                category: tx.category,
            };

            let liability_posting = LiabilityPosting {
                account: liability_account,
                amount: -tx.amount as f64,
                currency: tx.currency.to_string(),
                description: to_description,
            };

            let asset_account = AssetAccount {
                account_type: AccountType::Assets,
                currency: tx.currency.to_string(),
                provider: "Monzo".to_string(),
                name: tx.account_name.to_string(),
            };

            let asset_posting = AssetPosting {
                account: asset_account,
                amount: tx.amount as f64,
                currency: tx.currency.clone(),
            };

            let postings = Postings {
                liability_posting,
                asset_posting,
            };

            let comment = Some(tx.notes.clone().unwrap());
            let date = tx.settled.unwrap_or(tx.created).date();

            let notes = format_note(
                &tx.merchant_name,
                &tx.currency,
                &tx.local_currency,
                &tx.local_amount,
            );

            let transaction = Transaction {
                comment,
                date,
                notes,
                postings,
            };

            directives.push(Directive::Transaction(transaction));
        }
    }

    for d in directives {
        println!("{}", d.to_formatted_string());
    }

    Ok(())
}

fn format_note(
    merchant_name: &Option<String>,
    currency: &str,
    local_currency: &str,
    local_amount: &i64,
) -> String {
    // formant merchant name
    let merchant = merchant_name.clone().unwrap_or(String::new());

    // format currency
    let currency = if currency == local_currency {
        String::new()
    } else {
        if let Some(iso_code) = iso::find(local_currency) {
            format!(" {}", Money::from_minor(*local_amount, iso_code))
        } else {
            format!(" {} {}", local_amount, local_currency)
        }
    };

    format!("{}{}", merchant, currency)
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
