use prover_sdk::sdk::ProverSDK;
use serde::Deserialize;
use serde_json::Value;

use crate::errors::ProveErrors;

#[derive(Deserialize)]
pub struct JobId {
    pub job_id: u64,
}

pub async fn fetch_job(sdk: ProverSDK, job: String) -> Result<String, ProveErrors> {
    let job: JobId = serde_json::from_str(&job)?;
    println!("Job ID: {}", job.job_id);
    sdk.sse(job.job_id).await?;
    let response = sdk.get_job(job.job_id).await?;
    let response = response.text().await?;
    let json_response: Value = serde_json::from_str(&response)?;
    if let Some(status) = json_response.get("status").and_then(Value::as_str) {
        if status == "Completed" {
            return Ok(json_response
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or("No result found")
                .to_string());
        } else {
            Err(ProveErrors::Custom(json_response.to_string()))
        }
    } else {
        Err(ProveErrors::Custom(json_response.to_string()))
    }
}
