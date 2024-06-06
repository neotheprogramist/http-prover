use std::collections::BTreeMap;

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use josekit::{jwk::alg::ec::EcKeyPair, jwt::JwtPayload};
use openssl::hash::hash;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::stack::Stack;
use openssl::x509::extension::SubjectAlternativeName;
use openssl::x509::X509NameBuilder;
use openssl::x509::X509Req;
use reqwest::{header, Client, Response};
use serde_json::{json, Value};

use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;

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
            format!("mailto:{}", contact_mail),
        )])),
    );
    let body = create_jws(nonce, payload, urls.new_account.clone(), ec_key_pair, None).unwrap();
    post(client, urls.new_account, body).await
}

pub async fn submit_order(
    client: &Client,
    urls: DirectoryUrls,
    identifiers: Vec<&str>,
    ec_key_pair: EcKeyPair,
    kid: String,
) -> reqwest::Response {
    let mut payload = JwtPayload::new();
    let nonce = new_nonce(&client, urls.clone().new_nonce).await;
    let _ = payload.set_claim(
        "identifiers",
        Some(serde_json::Value::Array(
            identifiers
                .iter()
                .map(|x| {
                    serde_json::json!({
                        "type": "dns",
                        "value": x
                    })
                })
                .collect::<Vec<serde_json::Value>>(),
        )),
    );
    let body = create_jws(
        nonce,
        payload,
        urls.new_order.clone(),
        ec_key_pair,
        Some(kid),
    )
    .unwrap();
    post(client, urls.new_order, body).await
}

pub async fn fetch_authorizations(response: Value) -> Vec<String> {
    let order = response;
    let authorizations: Vec<String> = order["authorizations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|authz| authz.as_str().unwrap().to_string())
        .collect();
    authorizations
}
pub async fn choose_challanges(authorizations: Vec<String>, challange_type: &str) -> Vec<String> {
    let client = Client::new();
    let mut challanges: Vec<String> = Vec::new();
    for authz in authorizations {
        let response = client.get(authz).send().await.unwrap();
        let authz = response.json::<Value>().await.unwrap();
        let challange = authz["challenges"]
            .as_array()
            .unwrap()
            .iter()
            .find(|challange| challange["type"] == challange_type)
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
pub async fn respond_to_challange(
    challange_url: String,
    ec_key_pair: EcKeyPair,
    kid: String,
) -> Response {
    let client = Client::new();
    let payload = JwtPayload::new();
    let nonce = new_nonce(&client, challange_url.clone()).await;
    let body = create_jws(
        nonce,
        payload,
        challange_url.clone(),
        ec_key_pair,
        Some(kid),
    )
    .unwrap();
    post(&client, challange_url, body).await
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
pub async fn fetch_order_status(client: &Client, order_url: &str) -> Result<Value, reqwest::Error> {
    let response = client.get(order_url).send().await?;
    response.json::<Value>().await
}
pub fn get_thumbprint(ec_key_pair: EcKeyPair) -> String {
    let jwk_json = ec_key_pair.to_jwk_public_key();
    let mut jwk_btree_map = BTreeMap::new();
    jwk_btree_map.insert("crv", jwk_json.curve().unwrap().to_string());
    jwk_btree_map.insert("kty", jwk_json.key_type().to_string());
    jwk_btree_map.insert(
        "x",
        jwk_json
            .parameter("x")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    );
    jwk_btree_map.insert(
        "y",
        jwk_json
            .parameter("y")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    );
    // Convert to canonical JSON string
    let sorted_jwk_json = json!(jwk_btree_map).to_string();
    let jwk_digest = hash(MessageDigest::sha256(), sorted_jwk_json.as_bytes()).unwrap();
    BASE64_URL_SAFE_NO_PAD.encode(&jwk_digest)
}
pub async fn order_finalization(
    csr: String,
    urls: DirectoryUrls,
    ec_key_pair: EcKeyPair,
    kid: String,
    finalization_url: String,
) -> Response {
    let client = Client::new();
    let mut payload = JwtPayload::new();
    let nonce = new_nonce(&client, urls.clone().new_nonce).await;
    let _ = payload.set_claim("csr", Some(serde_json::Value::String(csr)));
    let body = create_jws(
        nonce,
        payload,
        finalization_url.clone(),
        ec_key_pair,
        Some(kid),
    )
    .unwrap();
    post(&client, finalization_url, body).await
}
pub fn get_key_authorization(token: String, ec_key_pair: EcKeyPair) -> String {
    let thumbprint = get_thumbprint(ec_key_pair);
    // Construct key authorization using the token and the thumbprint
    let key_authorization = format!("{}.{}", token, thumbprint);
    // Compute SHA-256 hash of the key authorization
    let key_auth_digest = hash(MessageDigest::sha256(), key_authorization.as_bytes()).unwrap();
    BASE64_URL_SAFE_NO_PAD.encode(&key_auth_digest)
}

pub fn generate_csr(domain: Vec<&str>) -> Result<String, openssl::error::ErrorStack> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let ec_key = EcKey::generate(&group)?;
    let pkey = PKey::from_ec_key(ec_key)?;
    // Build the X509 request with the domain name
    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_nid(openssl::nid::Nid::COMMONNAME, domain[0])?;
    let name = name_builder.build();

    let mut san_builder = SubjectAlternativeName::new();
    for d in domain {
        san_builder.dns(d);
    }

    let mut req_builder = X509Req::builder()?;
    req_builder.set_subject_name(&name)?;
    req_builder.set_pubkey(&pkey)?;
    // Add the SAN extension (Subject Alternative Name
    let context = req_builder.x509v3_context(None);
    let san_extension = san_builder.build(&context)?;
    let mut stack = Stack::new()?;
    stack.push(san_extension)?;
    req_builder.add_extensions(&stack)?;
    req_builder.sign(&pkey, openssl::hash::MessageDigest::sha256())?;

    let req = req_builder.build();
    let csr_der = req.to_der()?;
    let csr_base64 = BASE64_URL_SAFE_NO_PAD.encode(&csr_der);
    Ok(csr_base64)
}
#[cfg(test)]
mod tests {
    use super::*;
    use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
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
        println!("{:?}", nonce);
        Ok(())
    }
    #[tokio::test]
    async fn test_new_account() -> Result<(), Box<dyn std::error::Error>> {
        let ec_key_pair = EcKeyPair::generate(EcCurve::P256)?;
        let client = Client::new();
        let urls = new_directory().await;
        let response = new_account(&client, urls, "mateo@gmail.com".to_string(), ec_key_pair).await;
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
