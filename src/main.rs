use anyhow::Error;
use monzo::client::MonzoClient;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = MonzoClient::new()?;

    let whoami = client.whoami().await?;

    println!("{:?}", whoami);

    Ok(())
}
