use super::MonzoClient;
use anyhow::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WhoAmI {
    pub authenticated: bool,
    pub client_id: String,
    pub user_id: String,
}

impl MonzoClient {
    pub async fn whoami(&self) -> Result<WhoAmI, Error> {
        let url = format!("{}ping/whoami", self.base_url);
        let response = self.client.get(&url).send().await?;
        let whoami: WhoAmI = Self::handle_response(response).await?;

        Ok(whoami)
    }
}

#[cfg(test)]
mod test {
    use crate::tests::test::get_client;

    #[tokio::test]
    #[ignore]
    async fn whoami_work() {
        let monzo = get_client();
        match monzo.whoami().await {
            Ok(who_am_i) => {
                println!("->> OK {:#?}", who_am_i);
            }
            Err(e) => {
                println!("->> FAIL {:?}", e);
            }
        }
        // assert!(who_am_i.authenticated);
    }
}
