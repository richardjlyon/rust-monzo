use anyhow::Error;
use monzo::client::MonzoClient;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let monzo = MonzoClient::new()?;

    // get accounts
    let accounts = monzo.accounts().await?;

    // get balances for accounts
    for account in accounts {
        println!("Getting balance for account: {}", account.id);
        let balance = monzo.balance(&account.id).await?;
        println!("{}: {:?}", account.description, balance);
    }

    // get transactions for each account

    // process transactions

    // write transactions to a file

    Ok(())
}
