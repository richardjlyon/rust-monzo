//! Beancount

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::{
    beancount::{
        Account as BeanAccount, AccountType, Beancount, Directive, Posting, Postings, Transaction,
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
    let start = NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let end = NaiveDateTime::parse_from_str("2024-01-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();
    let date_ranges = date_ranges(start, end, 30);

    for (since, before) in date_ranges {
        let transactions = service.read_transactions_for_dates(since, before).await?;
        for tx in transactions {
            let from_amount = 0.0;
            let from_currency = "".to_string();
            let from_description = "".to_string();

            let to_amount = 0.0;
            let to_currency = "".to_string();
            let to_description = "".to_string();

            let from_account = BeanAccount {
                account_type: AccountType::Assets,
                currency: "XXX".to_string(),
                provider: "Monzo".to_string(),
                name: "XXX".to_string(),
            };

            let to_account = BeanAccount {
                account_type: AccountType::Assets,
                currency: "XXX".to_string(),
                provider: "Monzo".to_string(),
                name: "XXX".to_string(),
            };

            let from_posting = Posting {
                account: from_account,
                amount: from_amount,
                currency: from_currency,
                description: from_description,
            };

            let to_posting = Posting {
                account: to_account,
                amount: to_amount,
                currency: to_currency,
                description: to_description,
            };

            let postings = Postings {
                from: from_posting,
                to: to_posting,
            };

            let transaction = Transaction {
                date: Utc::now().date_naive(),
                description: "XXX".to_string(),
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

async fn monzo_assets(pool: DatabasePool) -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let acc_service = SqliteAccountService::new(pool.clone());
    let mut directives: Vec<Directive> = Vec::new();
    let accounts = acc_service.read_accounts().await?;

    for account in accounts {
        let beanaccount = BeanAccount {
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
        let beanaccount = BeanAccount {
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
        let beanaccount = BeanAccount {
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
        let beanaccount = BeanAccount {
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
        let beanaccount = BeanAccount {
            name: account.name,
            account_type: AccountType::Equities,
            currency: account.currency,
            provider: account.provider,
        };
        directives.push(Directive::Open(open_date, beanaccount, None));
    }

    Ok(directives)
}
