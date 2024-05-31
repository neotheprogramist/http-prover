use reqwest::{header, Client};
use serde_json::Value;

use super::types::DirectoryUrls;
const JOSE_JSON: &str = "application/jose+json";
const REPLAY_NONCE: &str = "replay-nonce";

pub async fn new_directory() -> DirectoryUrls {
    let client = Client::new();
    let directory_url = "https://acme-staging-v02.api.letsencrypt.org/directory";
    let response = client.get(directory_url).send().await.unwrap();
    let dir: Value = response.json().await.unwrap();

    DirectoryUrls {
        new_nonce: dir["newNonce"].as_str().unwrap().to_string(), // Extract as str and convert to String
        new_account: dir["newAccount"].as_str().unwrap().to_string(),
        new_order: dir["newOrder"].as_str().unwrap().to_string(),
        new_authz: dir["newAuthz"].as_str().map(String::from), // Map to convert Option<&str> to Option<String>
        revoke_cert: dir["revokeCert"].as_str().map(String::from),
        key_change: dir["keyChange"].as_str().map(String::from),
    }
}

pub async fn new_nonce(client: &Client, url_value: String) -> String {
    let response = client.head(url_value).send().await.unwrap();
    let nonce = response
        .headers()
        .get(REPLAY_NONCE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    nonce
}

pub async fn new_account(client: &Client, url_value: String, body: String) -> reqwest::Response {
    let response = client
        .post(url_value)
        .header(header::CONTENT_TYPE, JOSE_JSON)
        .body(body)
        .send()
        .await
        .unwrap();
    response
}
pub async fn submit_order(client: &Client, url_value: String, body: String) -> Value {
    let response = client
        .post(url_value)
        .header(header::CONTENT_TYPE, JOSE_JSON)
        .body(body)
        .send()
        .await
        .unwrap();
    let order: Value = response.json().await.unwrap();
    order
}
pub async fn challange_http01() {

    //TODO
}
#[cfg(test)]
mod tests {
    use crate::cert::create_jws::create_jws;
    use josekit::{
        jwk::alg::ec::{EcCurve, EcKeyPair},
        jwt::JwtPayload,
    };

    use super::*;
    #[tokio::test]
    async fn test_new_directory() -> Result<(), Box<dyn std::error::Error>> {
        let urls = new_directory().await;
        assert_eq!(
            urls.new_nonce,
            "https://acme-v02.api.letsencrypt.org/acme/new-nonce"
        );
        assert_eq!(
            urls.new_account,
            "https://acme-v02.api.letsencrypt.org/acme/new-acct"
        );
        assert_eq!(
            urls.new_order,
            "https://acme-v02.api.letsencrypt.org/acme/new-order"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_new_nonce() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();
        let urls = new_directory().await;
        let nonce = new_nonce(&client, urls.new_nonce).await;
        Ok(())
    }
    #[tokio::test]
    async fn test_new_account() -> Result<(), Box<dyn std::error::Error>> {
        let ec_key_pair = EcKeyPair::generate(EcCurve::P256)?;
        let mut _payload = JwtPayload::new();
        let client = Client::new();
        let urls = new_directory().await;
        let nonce = new_nonce(&client, urls.new_nonce).await;
        let _ = _payload.set_claim("termsOfServiceAgreed", Some(serde_json::Value::Bool(true)));
        _ = _payload.set_claim(
            "contact",
            Some(serde_json::Value::Array(vec![serde_json::Value::String(
                "mailto:cert-admin@gmail.com".to_string(),
            )])),
        );

        let jws = create_jws(nonce, _payload, urls.new_account.clone(), ec_key_pair,None)?;
        let response = new_account(&client, urls.new_account, jws).await;
        let mut accountlink: String = "".to_string();
        if response.status().is_success() {
            // Attempt to extract the "location" header
            if let Some(location) = response.headers().get("location") {
                accountlink = location.to_str()?.to_string();
            } else {
                println!("Account URL not found")
            }

        } else {
            println!("Account creation failed: {:?}", response.text().await?);
        }
        println!("{:?}", accountlink);
        Ok(())
    }
    #[tokio::test]
    async fn test_submit_order() -> Result<(), Box<dyn std::error::Error>> {
        let ec_key_pair = EcKeyPair::generate(EcCurve::P256)?;
        let mut _payload = JwtPayload::new();
        let client = Client::new();
        let urls = new_directory().await;
        let nonce = new_nonce(&client, urls.new_nonce).await;
        let _ = _payload.set_claim(
            "identifiers",
            Some(serde_json::Value::Array(vec![serde_json::json!({
                "type": "dns",
                "value": "blabla.visoft.dev"
            })])),
        );
        let jws = create_jws(nonce, _payload, urls.new_order.clone(), ec_key_pair,None)?;
        let response = submit_order(&client, urls.new_order, jws).await;
        println!("{:?}", response.to_string());
        Ok(())
    }
}
