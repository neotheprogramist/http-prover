use common::models::{JobResponse, ProverResult};
use prover_sdk::sdk::ProverSDK;

pub async fn fetch_job(sdk: ProverSDK, job: u64) -> Option<ProverResult> {
    println!("Job ID: {}", job);
    sdk.sse(job).await.unwrap();
    let response = sdk.get_job(job).await.unwrap();
    let response = response.text().await.unwrap();
    let json_response: JobResponse = serde_json::from_str(&response).unwrap();

    if let JobResponse::Completed { result, .. } = json_response {
        return Some(result);
    }
    None
}
