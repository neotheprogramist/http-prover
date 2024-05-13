use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use axum::Json;
use podman::runner::Runner;
use super::prove_input::ProveInput;

pub async fn root(
    _claims: Claims,
    Json(program_input): Json<ProveInput>,
) -> Result<String, ProveError> {
    let runner = podman::runner::PodmanRunner::new("docker.io/chudas/stone5-poseidon3:latest");
    let v = serde_json::to_string(&program_input)?;
    let result: String = runner.run(&v).await?;
    let proof: serde_json::Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    Ok(final_result)
}