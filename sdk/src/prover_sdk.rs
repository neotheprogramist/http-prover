use crate::prove_sdk_builder::ProverSDKBuilder;
use crate::ProverSdkErrors;
use reqwest::Client;
use serde_json::Value;

pub struct ProverSDK {
    pub client: Client,
    pub url_prover: String,
}

impl ProverSDK {
    pub fn new(url_auth: &str, url_prover: &str) -> ProverSDKBuilder {
        ProverSDKBuilder::new(url_auth, url_prover)
    }

    pub async fn prove(&self, data: Value) -> Result<String, ProverSdkErrors> {
        let response = self
            .client
            .post(&self.url_prover)
            .json(&data)
            .send()
            .await?;
        let response_data = response.text().await?;
        println!("{}", response_data);
        Ok(response_data)
    }
}
