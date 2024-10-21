use crate::{access_key::ProverAccessKey, errors::SdkErrors, sdk_builder::ProverSDKBuilder};
use common::{
    prover_input::{Cairo0ProverInput, CairoProverInput, PieProverInput, ProverInput},
    requests::AddKeyRequest,
};
use ed25519_dalek::{ed25519::signature::SignerMut, VerifyingKey};
use futures::StreamExt;
use reqwest::{Client, Response};
use serde::Deserialize;
use url::Url;
#[derive(Debug, Clone)]
/// ProverSDK is a struct representing a client for interacting with the Prover service.
pub struct ProverSDK {
    pub client: Client,
    pub prover_cairo0: Url,
    pub prover_cairo: Url,
    pub pie: Url,
    pub verify: Url,
    pub get_job: Url,
    pub register: Url,
    pub sse: Url,
    pub authority: ProverAccessKey,
}

#[derive(Deserialize)]
pub struct JobId {
    pub job_id: u64,
}

impl ProverSDK {
    pub async fn new(url: Url, access_key: ProverAccessKey) -> Result<Self, SdkErrors> {
        let url = if !url.as_str().ends_with('/') {
            let mut url_with_slash = url.clone();
            url_with_slash.set_path(&format!("{}/", url.path()));
            url_with_slash
        } else {
            url
        };
        let auth_url = url.join("auth")?;
        ProverSDKBuilder::new(auth_url, url)
            .auth(access_key)
            .await?
            .build()
    }

    pub async fn prove_cairo0(&self, data: Cairo0ProverInput) -> Result<u64, SdkErrors> {
        self.prove(ProverInput::Cairo0(data), self.prover_cairo0.clone())
            .await
    }

    pub async fn prove_cairo(&self, data: CairoProverInput) -> Result<u64, SdkErrors> {
        self.prove(ProverInput::Cairo(data), self.prover_cairo.clone())
            .await
    }
    pub async fn prove_pie(&self, data: PieProverInput) -> Result<u64, SdkErrors> {
        self.prove(ProverInput::Pie(data), self.pie.clone()).await
    }

    async fn prove(&self, data: ProverInput, url: Url) -> Result<u64, SdkErrors> {
        let response = self
            .client
            .post(url.clone())
            .json(&data.to_json_value())
            .send()
            .await?;

        if !response.status().is_success() {
            let response_data: String = response.text().await?;
            tracing::error!("{}", response_data);
            return Err(SdkErrors::ProveResponseError(response_data));
        }
        let response_data = response.text().await?;
        let job = serde_json::from_str::<JobId>(&response_data)?;
        Ok(job.job_id)
    }
    pub async fn verify(self, proof: String) -> Result<String, SdkErrors> {
        let response = self
            .client
            .post(self.verify.clone())
            .json(&proof)
            .send()
            .await?;
        let response_data = response.text().await?;
        Ok(response_data)
    }
    pub async fn get_job(&self, job_id: u64) -> Result<Response, SdkErrors> {
        let url = format!("{}/{}", self.get_job.clone().as_str(), job_id);
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let response_data: String = response.text().await?;
            tracing::error!("{}", response_data);
            return Err(SdkErrors::GetJobResponseError(response_data));
        }
        Ok(response)
    }
    pub async fn register(&mut self, key: VerifyingKey) -> Result<(), SdkErrors> {
        let signature = self.authority.0.sign(key.as_bytes());
        let request = AddKeyRequest {
            signature,
            new_key: key,
            authority: self.authority.0.verifying_key(),
        };
        let response = self
            .client
            .post(self.register.clone())
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(SdkErrors::RegisterResponseError(format!(
                "Failed to register key with status code: {}",
                response.status(),
            )));
        }
        Ok(())
    }
    pub async fn sse(&self, job_id: u64) -> Result<(), SdkErrors> {
        let url = format!("{}?job_id={}", self.sse.clone().as_str(), job_id);
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(SdkErrors::SSEError(format!(
                "Failed to get SSE with status code: {}",
                response.status(),
            )));
        }

        let mut stream = response.bytes_stream();
        while let Some(_item) = stream.next().await {}
        Ok(())
    }
}
