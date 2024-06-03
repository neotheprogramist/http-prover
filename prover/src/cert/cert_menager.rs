use josekit::{jwk::alg::ec::EcKeyPair, jwt::JwtPayload};
use reqwest::{get, header, Client, Response};
use serde_json::Value;

use super::{create_jws::create_jws, types::DirectoryUrls};
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

pub async fn new_account(
    client: &Client,
    urls: DirectoryUrls,
    contact_mail: String,
    ec_key_pair: EcKeyPair,
) -> reqwest::Response {
    // "termsOfServiceAgreed": true,
    // "contact": ["mailto:mail@example.com"]
    let mut payload = JwtPayload::new();
    let nonce = new_nonce(&client, urls.clone().new_nonce).await;
    let _ = payload.set_claim("termsOfServiceAgreed", Some(serde_json::Value::Bool(true)));
    let _ = payload.set_claim(
        "contact",
        Some(serde_json::Value::Array(vec![serde_json::Value::String(
            format!("mailto:{}",contact_mail),
        )])),
    );
    let body = create_jws(nonce, payload, urls.new_account.clone(), ec_key_pair, None).unwrap();
    post(client, urls.new_account, body).await
}

pub async fn submit_order(client: &Client, urls: DirectoryUrls,identifiers: Vec<String>,ec_key_pair: EcKeyPair,kid:String) -> reqwest::Response {
    // "identifiers": [
    //      { "type": "dns", "value": "www.example.org" },
    //      { "type": "dns", "value": "example.org" }
    //    ],
    let mut payload = JwtPayload::new();
    let nonce = new_nonce(&client, urls.clone().new_nonce).await;
    let _ = payload.set_claim("identifiers", Some(serde_json::Value::Array(identifiers.iter().map(|x| serde_json::json!({
        "type": "dns",
        "value": x
    })).collect::<Vec<serde_json::Value>>())));
    let body = create_jws(nonce, payload, urls.new_order.clone(), ec_key_pair, Some(kid)).unwrap();
    post(client, urls.new_order, body).await
}

pub async fn fetch_authorizations(response: Response) -> Vec<String> {
    let order = response.json::<Value>().await.unwrap();
    let authorizations: Vec<String> = order["authorizations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|authz| authz.as_str().unwrap().to_string())
        .collect();
    authorizations
}
pub async fn fetch_challanges(authorizations: Vec<String>) -> Vec<String> {
    let client = Client::new();
    let mut challanges: Vec<String> = Vec::new();
    for authz in authorizations {
        let response = client.get(authz).send().await.unwrap();
        let authz = response.json::<Value>().await.unwrap();
        let challange = authz["challenges"]
            .as_array()
            .unwrap()
            .iter()
            .find(|challange| challange["type"] == "http-01")
            .unwrap();
        challanges.push(challange["url"].as_str().unwrap().to_string());
    }
    challanges
}
pub async fn get_challanges_tokens(challanges: Vec<String>) -> Vec<String> {
    let client = Client::new();
    let mut details: Vec<String> = Vec::new();
    for challange in challanges {
        let response = client.get(challange).send().await.unwrap();
        let detail = response.json::<Value>().await.unwrap();
        details.push(detail["token"].as_str().unwrap().to_string());
    }
    details
}

pub async fn post(client: &Client, url_value: String, body: String) -> reqwest::Response {
    let response = client
        .post(url_value)
        .header(header::CONTENT_TYPE, JOSE_JSON)
        .body(body)
        .send()
        .await
        .unwrap();
    response
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
        let client = Client::new();
        let urls = new_directory().await;
        let response = new_account(&client,urls,"mateo@gmail.com".to_string(),ec_key_pair).await;
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
}
