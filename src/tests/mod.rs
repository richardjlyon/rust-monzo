#[cfg(test)]
pub mod test {
    use crate::client::MonzoClient;

    pub fn get_client() -> MonzoClient {
        match MonzoClient::new() {
            Ok(client) => client,
            Err(_) => panic!("Error creating client"),
        }
    }
}
