use std::time::Duration;

use prover_sdk::{sdk::ProverSDK, JobResponse, ProverResult};
use serde_json::Value;
use tokio::time::sleep;
use tracing::info;

use crate::errors::ProveErrors;

pub async fn fetch_job_sse(sdk: ProverSDK, job: u64) -> Result<ProverResult, ProveErrors> {
    info!("Job ID: {}", job);
    sdk.sse(job).await?;
    info!("Job completed");
    let response = sdk.get_job(job).await?;
    let response = response.text().await?;
    let json_response: JobResponse = serde_json::from_str(&response).unwrap();
    if let JobResponse::Completed { result, .. } = json_response {
        return Ok(result);
    }
    Err(ProveErrors::Custom("Job failed".to_string()))
}
pub async fn fetch_job_polling(sdk: ProverSDK, job: u64) -> Result<ProverResult, ProveErrors> {
    info!("Fetching job: {}", job);
    let mut counter = 0;
    loop {
        let response = sdk.get_job(job).await?;
        let response = response.text().await?;
        let json_response: Value = serde_json::from_str(&response)?;
        if let Some(status) = json_response.get("status").and_then(Value::as_str) {
            match status {
                "Completed" => {
                    let json_response: JobResponse = serde_json::from_str(&response).unwrap();
                    if let JobResponse::Completed { result, .. } = json_response {
                        return Ok(result);
                    }
                }
                "Pending" | "Running" => {
                    info!("Job is still in progress. Status: {}", status);
                    info!(
                        "Time passed: {} Waiting for 10 seconds before retrying...",
                        counter * 10
                    );
                    counter += 1;
                    sleep(Duration::from_secs(10)).await;
                }
                _ => {
                    return Err(ProveErrors::Custom(json_response.to_string()));
                }
            }
        }
    }
}
