use super::{ErrorJson, MonzoClient, MonzoClientError};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WhoAmI {
    pub authenticated: bool,
    pub client_id: String,
    pub user_id: String,
}

impl MonzoClient {
    pub async fn whoami(&self) -> Result<WhoAmI, MonzoClientError> {
        let url = format!("{}ping/whoami", self.base_url);
        let response = self.client.get(&url).send().await?;

        match response.status().is_success() {
            true => {
                let success_json = response.json::<WhoAmI>().await?;
                Ok(success_json)
            }
            false => {
                let error_json = response.json::<ErrorJson>().await?;
                Err(MonzoClientError::AuthorisationFailure(error_json))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tests::test::get_client;

    #[tokio::test]
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
