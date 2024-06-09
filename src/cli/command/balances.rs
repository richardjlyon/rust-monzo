use anyhow::Error;
use rusty_money::{iso, Money};

use crate::client::MonzoClient;

pub async fn balances() -> Result<(), Error> {
    let monzo = MonzoClient::new()?;

    let mut balance_total = 0;

    println!("{:>42}", "BALANCES");
    println!("------------------------------------------");

    // Display accounts
    for account in monzo.accounts().await? {
        let balance = monzo.balance(&account.id).await?;
        balance_total = balance_total + balance.balance;

        let iso_code = iso::find(&balance.currency).unwrap();
        let balance_fmt = Money::from_minor(balance.balance, iso_code).to_string();
        let spend_today_fmt = Money::from_minor(balance.spend_today, iso_code).to_string();

        println!(
            "{:<8} ({}) : {:>11} {:>8}",
            account.owner_type, account.account_number, balance_fmt, spend_today_fmt,
        );

        // Display pots
        for pot in monzo.pots(&account.id).await? {
            if pot.deleted {
                continue;
            }
            balance_total = balance_total + pot.balance;
            let iso_code = iso::find(&balance.currency).unwrap();
            let balance_fmt = Money::from_minor(pot.balance, iso_code).to_string();

            println!("- {:<18}: {:>11}", pot.name.to_lowercase(), balance_fmt);
        }
    }
    println!("------------------------------------------");
    println!(
        "Total: {:>26}",
        Money::from_minor(balance_total, iso::GBP).to_string() // FIXME: Use the account currency
    );

    Ok(())
}
