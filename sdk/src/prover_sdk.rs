use crate::models::JWTResponse;
use crate::ProverSdkErrors;
use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use ed25519_dalek::{Signature, Signer};
use reqwest::cookie::Jar;
use reqwest::Client;
use reqwest::Url;
use serde_json::{json,Value};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProverSDKBuilder {
    client: Client,
    url_auth: String,
    url_prover: String,
    signing_key: Option<SigningKey>,
    jwt_token: Option<String>,
}
pub struct ProverSDK {
    client: Client,
    url_prover:String,
}

impl ProverSDKBuilder {
    pub fn new() -> Self {
        ProverSDKBuilder {
            client: Client::new(),
            url_auth: "http://localhost:7003/auth".to_string(),
            url_prover: "http://localhost:7003/prove/state-diff-commitment".to_string(),
            signing_key: None,
            jwt_token: None,
        }
    }

    pub async fn auth(mut self, private_key_hex: &str) -> Result<Self, ProverSdkErrors> {
        // Convert the hexadecimal private key string into bytes
        let private_key_bytes = hex::decode(private_key_hex).expect("Failed to decode hexadecimal string");
        let mut private_key_array = [0u8; 32];
        private_key_array.copy_from_slice(&private_key_bytes);
        let signing_key = SigningKey::from_bytes(&private_key_array);

        self.signing_key = Some(signing_key);
        let jwt_response = self.get_jwt_token().await?;
        self.jwt_token = Some(jwt_response.jwt_token);
        Ok(self)
    }

    async fn get_jwt_token(&self) -> Result<JWTResponse, ProverSdkErrors> {
        let signing_key = self.signing_key.as_ref().ok_or(ProverSdkErrors::SigningKeyNotFound)?;
        let public_key = signing_key.verifying_key();

        let nonce = self.get_nonce(&public_key).await?;

        let signed_nonce = signing_key.sign(nonce.as_bytes());

        self.validate_signature(&public_key, &nonce, &signed_nonce).await
    }

    async fn get_nonce(&self, public_key: &VerifyingKey) -> Result<String, ProverSdkErrors> {
        let url_with_params = format!(
            "{}?public_key={}",
            &self.url_auth,
            bytes_to_hex_string(public_key.as_bytes())
        );

        let response = self.client.get(&url_with_params).send().await?;

        let response_text = response.text().await?;
        let json_body: Value = serde_json::from_str(&response_text)?;

        let nonce = json_body["nonce"].as_str()
            .ok_or(ProverSdkErrors::NonceNotFound)?
            .to_string();

        Ok(nonce)
    }

    async fn validate_signature(&self, public_key: &VerifyingKey, nonce: &String, signed_nonce: &Signature) -> Result<JWTResponse, ProverSdkErrors> {
        let data = json!({
            "public_key": bytes_to_hex_string(&public_key.to_bytes()),
            "nonce": nonce,
            "signature": bytes_to_hex_string(&signed_nonce.to_bytes()),
        });

        let response = self.client.post(&self.url_auth)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&data)
            .send()
            .await?;

        let json_body: Value = response.json().await?;
        let jwt_token = json_body["jwt_token"].as_str()
            .ok_or(ProverSdkErrors::JwtTokenNotFound)?
            .to_string();
        let expiration = json_body["expiration"].as_u64()
            .ok_or(ProverSdkErrors::ExpirationNotFound)?;

        Ok(JWTResponse {
            jwt_token,
            expiration,
        })
    }

    pub fn build(self) -> Result<ProverSDK, ProverSdkErrors> {
        let _signing_key = self.signing_key.ok_or(ProverSdkErrors::SigningKeyNotFound)?;
        let jwt_token = self.jwt_token.ok_or(ProverSdkErrors::JwtTokenNotFound)?;

        let url_prover = Url::parse(&self.url_prover)?;

        let jar = Jar::default();
        jar.add_cookie_str(&format!("jwt_token={}; HttpOnly; Secure; Path=/", jwt_token), &url_prover);

        let client = reqwest::Client::builder()
            .cookie_provider(Arc::new(jar))
            .build()?;

        Ok(ProverSDK {
            client,
            url_prover: self.url_prover,
        })
    }
}

impl ProverSDK {
    pub fn new() -> ProverSDKBuilder {
        ProverSDKBuilder::new()
    }

    pub async fn prove(&self, data: Value) -> Result<String, ProverSdkErrors> {
        let response = self.client.post(&self.url_prover).json(&data).send().await?;
        let response_data = response.text().await?;
        println!("{}", response_data);
        Ok(response_data)
    }
}

// Convert byte array to hexadecimal string
fn bytes_to_hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}
