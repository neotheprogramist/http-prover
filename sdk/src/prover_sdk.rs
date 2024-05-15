use crate::errors::ProverSdkErrors;
use crate::prove_sdk_builder::ProverSDKBuilder;

use common::ProverInput;
use reqwest::Client;
use url::Url;

/// ProverSDK is a struct representing a client for interacting with the Prover service.
pub struct ProverSDK {
    pub client: Client,
    pub prover: Url,
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
    pub fn new(auth: Url, prover: Url) -> ProverSDKBuilder {
        ProverSDKBuilder::new(auth, prover)
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
    pub async fn prove<T>(&self, data: T) -> Result<String, ProverSdkErrors>
    where
        T: ProverInput + Send + 'static,
    {
        let response = match self
            .client
            .post(self.prover.clone())
            .json(&data.serialize())
            .send()
            .await
        {
            Ok(response) => response,
            Err(request_error) => {
                return Err(ProverSdkErrors::ProveRequestFailed(format!(
                    "Failed to send HTTP request to URL: {}. Error: {}",
                    self.prover, request_error
                )));
            }
        };

        if !response.status().is_success() {
            return Err(ProverSdkErrors::ProveResponseError(format!(
                "Received unsuccessful status code ({}) from URL: {}",
                response.status(),
                self.prover
            )));
        }

        let response_data = match response.text().await {
            Ok(response_text) => response_text,
            Err(text_error) => {
                return Err(ProverSdkErrors::ProveResponseError(format!(
                    "Failed to read response text from URL: {}. Error: {}",
                    self.prover, text_error
                )));
            }
        };

        Ok(response_data)
    }
}
