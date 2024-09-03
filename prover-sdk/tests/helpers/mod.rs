use prover_sdk::sdk::ProverSDK;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct JobId {
    pub job_id: u64,
}
pub async fn fetch_job(sdk: ProverSDK, job: String) -> String {
    let job: JobId = serde_json::from_str(&job).unwrap();
    println!("Job ID: {}", job.job_id);
    sdk.sse(job.job_id).await.unwrap();
    let response = sdk.get_job(job.job_id).await.unwrap();
    let response = response.text().await.unwrap();
    let json_response: Value = serde_json::from_str(&response).unwrap();
    return json_response
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or("No result found")
        .to_string();
}
