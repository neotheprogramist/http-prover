use prover_sdk::sdk::ProverSDK;
use serde_json::Value;

pub async fn fetch_job(sdk: ProverSDK, job: u64) -> String {
    println!("Job ID: {}", job);
    sdk.sse(job).await.unwrap();
    let response = sdk.get_job(job).await.unwrap();
    let response = response.text().await.unwrap();
    let json_response: Value = serde_json::from_str(&response).unwrap();
    return json_response
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or("No result found")
        .to_string();
}
