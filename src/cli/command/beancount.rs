//! Beancount

use chrono::{DateTime, Utc};

use crate::{
    beancount::{Account as BeanAccount, AccountType, Beancount, Directive},
    error::AppErrors as Error,
    model::{
        account::{Service as AccountService, SqliteAccountService},
        pot::{Service as PotService, SqlitePotService},
        transaction::SqliteTransactionService,
        DatabasePool,
    },
};

pub async fn beancount(pool: DatabasePool) -> Result<(), Error> {
    let mut directives: Vec<Directive> = Vec::new();

    // Open assets
    directives.push(Directive::Comment("assets".to_string()));
    directives.extend(monzo_assets(pool.clone()).await?);
    directives.extend(monzo_pots(pool.clone()).await?);
    directives.extend(config_assets().await?);

    // Open liabilities
    directives.push(Directive::Comment("liabilities".to_string()));
    directives.extend(config_liabilities().await?);

    // Open equity accounts
    directives.push(Directive::Comment("equities".to_string()));
    directives.extend(config_equities().await?);

    // Banking - Get January `Personal` transactions

    let _service = SqliteTransactionService::new(pool);

    let _since = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let _before = DateTime::parse_from_rfc3339("2024-01-31T23:59:59Z")
        .unwrap()
        .with_timezone(&Utc);

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

async fn config_assets() -> Result<Vec<Directive>, Error> {
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

async fn config_liabilities() -> Result<Vec<Directive>, Error> {
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

async fn config_equities() -> Result<Vec<Directive>, Error> {
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
