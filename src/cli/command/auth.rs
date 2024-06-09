use std::collections::HashMap;

use url::Url;
use uuid::Uuid;

use crate::configuration::get_configuration;

pub async fn auth() {
    let config = get_configuration().unwrap();
    open_login_page(
        &config.oath_credentials.client_id,
        &config.oath_credentials.redirect_uri,
    )
    .await;
}

async fn open_login_page(client_id: &str, redirect_uri: &str) {
    let state = Uuid::new_v4().to_string();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("response_type", "code");
    params.insert("state", &state);

    let base_url = "https://auth.monzo.com/";
    let url = generate_url(base_url, &params);

    webbrowser::open(&url).expect("Failed to open browser");
}

// Generate the login URL
fn generate_url(base_url: &str, params: &HashMap<&str, &str>) -> String {
    let mut url = Url::parse(base_url).expect("Invalid base URL");
    for (key, value) in params {
        url.query_pairs_mut().append_pair(key, value);
    }
    url.to_string()
}
