//! Get balances
//!
//! This command will fetch the balances of all accounts
//! and print them to the console.

use rusty_money::{iso, Money};

use crate::client::Monzo;
use crate::error::AppErrors as Error;

/// Get balances
///
/// # Errors
/// Will return errors if the Monzo API cannot be reached.
///
pub async fn balances() -> Result<(), Error> {
    let monzo = Monzo::new()?;

    let mut balance_total = 0;

    println!("{:>44}", "BALANCES");
    println!("--------------------------------------------");

    // Display accounts
    for account in monzo.accounts().await? {
        let balance = monzo.balance(&account.id).await?;
        balance_total += balance.balance;

        let Some(iso_code) = iso::find(&balance.currency) else {
            return Err(Error::CurrencyNotFound(balance.currency));
        };
        let balance_fmt = Money::from_minor(balance.balance, iso_code).to_string();
        let spend_today_fmt = Money::from_minor(balance.spend_today, iso_code).to_string();

        println!(
            "{:<8} ({}) : {:>11} {:>10}",
            account.owner_type, account.account_number, balance_fmt, spend_today_fmt,
        );

        // Display pots
        for pot in monzo.pots(&account.id).await? {
            if pot.deleted {
                continue;
            }
            balance_total += pot.balance;
            let Some(iso_code) = iso::find(&balance.currency) else {
                return Err(Error::CurrencyNotFound(balance.currency));
            };
            let balance_fmt = Money::from_minor(pot.balance, iso_code).to_string();

            println!("- {:<18}: {:>11}", pot.name.to_lowercase(), balance_fmt);
        }
    }
    println!("--------------------------------------------");
    println!(
        "Total: {:>26}",
        Money::from_minor(balance_total, iso::GBP).to_string() // FIXME: Use the account currency
    );

    Ok(())
}
