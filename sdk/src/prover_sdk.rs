use crate::prove_sdk_builder::ProverSDKBuilder;
use crate::ProverSdkErrors;
use reqwest::Client;
use serde_json::Value;

/// ProverSDK is a struct representing a client for interacting with the Prover service.
pub struct ProverSDK {
    pub client: Client,
    pub url_prover: String,
}

impl ProverSDK {
    /// Creates a new ProverSDK instance.
    ///
    /// # Arguments
    ///
    /// * `url_auth` - The URL of the authentication service.
    /// * `url_prover` - The URL of the Prover service.
    ///
    /// # Returns
    ///
    /// Returns a `ProverSDKBuilder` which can be used to further configure the ProverSDK.
    pub fn new(url_auth: &str, url_prover: &str) -> ProverSDKBuilder {
        ProverSDKBuilder::new(url_auth, url_prover)
    }

    /// Proves the provided data to the Prover service.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be proved, in JSON format.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a string representing the response from the Prover service
    /// if successful, or a `ProverSdkErrors` if an error occurs.
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