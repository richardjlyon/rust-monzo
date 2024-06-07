use super::{ErrorJson, MonzoClientError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

/// Structure for representing the components of the Oath client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OathCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// Structure for representing the components of the authcode request response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthCodeResponse {
    #[serde(rename = "code")]
    pub auth_code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct AuthRequest {
    pub oath_credentials: OathCredentials,
    pub auth_code: AuthCodeResponse,
}

/// Structure for representing the components of the access token request response
#[derive(Debug, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub client_id: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub token_type: String,
    pub user_id: String,
}

// fn generate_url(base_url: &str, params: &HashMap<&str, &str>) -> String {
//     let mut url = Url::parse(base_url).expect("Invalid base URL");
//     for (key, value) in params {
//         url.query_pairs_mut().append_pair(key, value);
//     }
//     url.to_string()
// }

/// Create the login URL and open a login page
// pub fn open_login_page(client_id: &str, redirect_uri: &str) {
//     let client = reqwest::Client::new();

//     let url = "https://auth.monzo.com/";
//     let state = Uuid::new_v4().to_string();

//     let mut params = HashMap::new();
//     params.insert("client_id", client_id);
//     params.insert("redirect_uri", redirect_uri);
//     params.insert("response_type", "code");
//     params.insert("state", &state);

//     let url = generate_url(&url, &params);

//     webbrowser::open(&url).expect("Failed to open browser");
// }

/// Exhange the auth code for an access token
pub async fn get_access_token(auth_request: &AuthRequest) -> Result<AccessToken, MonzoClientError> {
    let client = reqwest::Client::new();

    let url = "https://api.monzo.com/oath2/token";

    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", &auth_request.oath_credentials.client_id);
    params.insert(
        "client_secret",
        &auth_request.oath_credentials.client_secret,
    );
    params.insert("redirect_uri", "http://localhost:8080/auth/callback");
    params.insert("code", &auth_request.auth_code.auth_code);

    let response = client.post(url).query(&params).send().await?;

    match response.status().is_success() {
        true => {
            let access_token = response.json::<AccessToken>().await?;
            Ok(access_token)
        }
        false => {
            let error_json = response.json::<ErrorJson>().await?;
            Err(MonzoClientError::AuthorisationFailure(error_json))
        }
    }
}

#[derive(Debug, Error)]
pub enum UriError {
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Missing parameter: {0}")]
    MissingParameter(&'static str),
}

impl AuthCodeResponse {
    /// Create a new instance of `RedirectUriComponents` from a URL string
    pub fn new(url_str: &str) -> Result<Self, UriError> {
        let url = Url::parse(url_str)?;
        let query_pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();

        let auth_code = query_pairs
            .get("code")
            .ok_or(UriError::MissingParameter("code"))?
            .clone();
        let state = query_pairs
            .get("state")
            .ok_or(UriError::MissingParameter("state"))?
            .clone();
        Ok(Self { auth_code, state })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::configuration::get_configuration;

    #[tokio::test]
    async fn extract_authcode_works() {
        // Arrange
        let token = "https://developers.monzo.com/login?code=XXXeyJhbGciOiJFUzI1NiIsInR5IkpXVCJ9.eyJlYiI6InBEQXdNRGYxZHdJOHQ5YWN6dE0vIiwianRpIjoiYXV0aHpjb2RlXzAwMDBBaWRzR2d4RGxPdFJ3YlVqU2IiLCJ0eXAiOiJhemMiLCJ2IjoiNiJ9.d81tSrMC4DyMnNzEINg8d_sfHlL6W_fjjTJhwnVxvt7LbqNEo3K_gQGZx3JAflAcE41loMu2K-nGISinoe3RVA&redirect=%2Fapi%2Fplayground&state=a1d77c29-bc27-42df-acb2-4d90712aa876";
        let expected_code = "XXXeyJhbGciOiJFUzI1NiIsInR5IkpXVCJ9.eyJlYiI6InBEQXdNRGYxZHdJOHQ5YWN6dE0vIiwianRpIjoiYXV0aHpjb2RlXzAwMDBBaWRzR2d4RGxPdFJ3YlVqU2IiLCJ0eXAiOiJhemMiLCJ2IjoiNiJ9.d81tSrMC4DyMnNzEINg8d_sfHlL6W_fjjTJhwnVxvt7LbqNEo3K_gQGZx3JAflAcE41loMu2K-nGISinoe3RVA";
        let expected_state = "a1d77c29-bc27-42df-acb2-4d90712aa876";

        // Act
        let auth_code = AuthCodeResponse::new(&token).unwrap();

        // Assert
        assert_eq!(auth_code.auth_code, expected_code);
        assert_eq!(auth_code.state, expected_state);

        println!("{:#?}", auth_code);
    }

    #[tokio::test]
    async fn get_auth_code_works() {
        let config = get_configuration().unwrap();
        let oath_credentials = config.oath_credentials;

        let auth_code =
            open_login_page(&oath_credentials.client_id, &oath_credentials.redirect_uri);
    }

    #[tokio::test]
    async fn get_access_token_works() {
        // Arrange
        let token = "http://localhost:8080/auth/callback?code=eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJlYiI6Im5CTmcrS3hLQ20yS0F5UTJGVjhyIiwianRpIjoiYXV0aHpjb2RlXzAwMDBBaWd4eVBPN1c5YnpWeXNlY3oiLCJ0eXAiOiJhemMiLCJ2IjoiNiJ9.PgZ9brMOFRBOQGI4R9PpMddXpEU8wajqptzUSzYspwCVK0X5EUwRaRQSkJ4KuwEsYVbjDy1pLVchhhYWJwYIQg&state=123456";
        let auth_code = AuthCodeResponse::new(&token).unwrap();

        let config = get_configuration().unwrap();
        let oath_credentials = config.oath_credentials.clone();

        let auth_request = super::AuthRequest {
            oath_credentials,
            auth_code,
        };

        // let auth_request = AuthRequest {
        //     auth_code: AuthCode {
        //         auth_code: "XXXeyJhbGciOiJFUzI1NiIsInR5IkpXVCJ9.eyJlYiI6InBEQXdNRGYxZHdJOHQ5YWN6dE0vIiwianRpIjoiYXV0aHpjb2RlXzAwMDBBaWRzR2d4RGxPdFJ3YlVqU2IiLCJ0eXAiOiJhemMiLCJ2IjoiNiJ9.d81tSrMC4DyMnNzEINg8d_sfHlL6W_fjjTJhwnVxvt7LbqNEo3K_gQGZx3JAflAcE41loMu2K-nGISinoe3RVA".to_string(),
        //         state: "a1d77c29-bc27-42df-acb2-4d90712aa876".to_string(),
        //     },
        //     oath_client: OathClient {
        //         id: "oauth2client_0000AidEQRuYYFNEoIC21z".to_string(),
        //         secret: "mnzconf.o1E6pPS2OAOUxhMsCshchASFuvw+Jv6/C2JZ2mrerWWJik7dKiOyyhRDN1dPV+X+/3ajLh2UQPinp0bdMLA8YQ==".to_string(),
        //     },
        // };

        let access_token = get_access_token(&auth_request).await.unwrap();

        println!("{:#?}", access_token);
    }
}
