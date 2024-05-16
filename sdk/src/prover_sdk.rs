use crate::errors::ProverSdkErrors;
use crate::prove_sdk_builder::ProverSDKBuilder;
use crate::ProverAccessKey;

use common::{AddAuthorizedRequest, ProverInput};
use ed25519_dalek::{ed25519::signature::SignerMut, VerifyingKey};
use reqwest::Client;
use url::Url;

/// ProverSDK is a struct representing a client for interacting with the Prover service.
pub struct ProverSDK {
    pub client: Client,
    pub prover: Url,
    pub authority: ProverAccessKey,
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
    pub async fn new(
        access_key: ProverAccessKey,
        auth: Url,
        prover: Url,
    ) -> Result<ProverSDK, ProverSdkErrors> {
        ProverSDKBuilder::new(auth, prover)
            .auth(access_key)
            .await?
            .build()
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

    pub async fn register(
        &mut self,
        key: VerifyingKey,
        url: Url,
    ) -> Result<String, ProverSdkErrors> {
        let signature = self.authority.0.sign(&key.to_bytes());
        let req = AddAuthorizedRequest {
            signature,
            authority: self.authority.0.verifying_key(),
            new_key: key,
        };

        let response = match self.client.post(url).json(&req).send().await {
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
