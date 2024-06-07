#[cfg(test)]
pub mod test {
    use crate::client::MonzoClient;
    use dotenv::dotenv;
    use std::env;

    pub fn get_client() -> MonzoClient {
        dotenv().ok();
        let access_token =
            env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN must be set in the .env file");

        match MonzoClient::new(access_token) {
            Ok(client) => client,
            Err(_) => panic!("Error creating client"),
        }
    }
}
