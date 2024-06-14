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
    // let _db = pool.db();
    let bc = Beancount::from_config()?;
    let acc_service = SqliteAccountService::new(pool.clone());
    let pot_service = SqlitePotService::new(pool.clone());
    let _tx_service = SqliteTransactionService::new(pool.clone());

    let mut directives: Vec<Directive> = Vec::new();

    // Open assets
    directives.push(Directive::Comment("assets".to_string()));

    // -- from database

    let accounts = acc_service.read_accounts().await?;
    for account in accounts {
        let beanaccount = BeanAccount {
            account_type: AccountType::Assets,
            currency: account.currency,
            provider: "Monzo".to_string(),
            name: account.owner_type,
        };
        directives.push(Directive::Open(bc.settings.start_date, beanaccount, None));
    }

    let pots = pot_service.read_pots().await?;
    for pot in pots {
        let beanaccount = BeanAccount {
            account_type: AccountType::Assets,
            currency: pot.currency,
            provider: "Monzo".to_string(),
            name: pot.name,
        };
        directives.push(Directive::Open(bc.settings.start_date, beanaccount, None));
    }

    // -- from configuration

    if bc.settings.assets.is_some() {
        for account in bc.settings.assets.unwrap() {
            let beanaccount = BeanAccount {
                name: account.name,
                account_type: AccountType::Assets,
                currency: account.currency,
                provider: "Monzo".to_string(),
            };
            directives.push(Directive::Open(bc.settings.start_date, beanaccount, None));
        }
    }

    // Open liabilities
    directives.push(Directive::Comment("liabilities".to_string()));

    for account in bc.settings.liabilities {
        directives.push(Directive::Open(
            bc.settings.start_date,
            account.clone(),
            None,
        ));
    }

    // Open equity accounts
    directives.push(Directive::Comment("equities".to_string()));

    for account in bc.settings.equities {
        let beanaccount = BeanAccount {
            account_type: AccountType::Equity,
            currency: account.currency,
            provider: "Monzo".to_string(),
            name: account.name,
        };
        directives.push(Directive::Open(bc.settings.start_date, beanaccount, None));
    }

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
