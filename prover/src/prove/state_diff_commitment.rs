use super::ProveError;
use crate::auth::jwt::Claims;
use podman::runner::Runner;

pub async fn root(_claims: Claims, program_input: String) -> Result<String, ProveError> {
    println!("PROVER CALLED");
    let runner = podman::runner::PodmanRunner::new("state-diff-commitment:latest");
    let v = program_input.to_string();
    let result: String = runner.run(&v).await?;
    let proof: serde_json::Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    Ok(final_result)
}
