use crate::errors::ProverSdkErrors;
use crate::prove_sdk_builder::ProverSDKBuilder;
use prover::prove::prove_input::ProveInput;
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

    /// Sends the provided data to the Prover service and returns the prover output.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be proved, in JSON format.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a string representing the response from the Prover service
    /// if successful, or a `ProverSdkErrors` if an error occurs.
    pub async fn prove(&self, data: ProveInput) -> Result<String, ProverSdkErrors> {
        let response = match self.client.post(&self.url_prover).json(&data).send().await {
            Ok(response) => response,
            Err(request_error) => {
                return Err(ProverSdkErrors::ProveRequestFailed(format!(
                    "Failed to send HTTP request to URL: {}. Error: {}",
                    &self.url_prover, request_error
                )));
            }
        };
        if !response.status().is_success() {
            return Err(ProverSdkErrors::ProveResponseError(format!(
                "Received unsuccessful status code ({}) from URL: {}",
                response.status(),
                &self.url_prover
            )));
        }
        let response_data = match response.text().await {
            Ok(response_text) => response_text,
            Err(text_error) => {
                return Err(ProverSdkErrors::ProveResponseError(format!(
                    "Failed to read response text from URL: {}. Error: {}",
                    &self.url_prover, text_error
                )));
            }
        };
        Ok(response_data)
    }
}
