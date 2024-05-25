use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use axum::Json;
use common::Cairo1ProverInput;
use podman::runner::Runner;

pub async fn root(
    _claims: Claims,
    Json(program_input): Json<Cairo1ProverInput>,
) -> Result<String, ProveError> {
    let runner = podman::runner::PodmanRunner::new("docker.io/neotheprogramist/stone-cairo:recursive");
    let v = serde_json::to_string(&program_input)?;
    let result: String = runner.run(&v).await?;
    let proof: serde_json::Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    Ok(final_result)
}
