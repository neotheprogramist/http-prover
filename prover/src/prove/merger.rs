use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use podman::runner::Runner;

pub async fn root(_claims: Claims, program_input: String) -> Result<String, ProveError> {
    let runner = podman::runner::PodmanRunner::new("docker.io/matzayonc/merger");
    let v = program_input.to_string();
    let result: String = runner.run(&v).await?;
    let proof: serde_json::Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    Ok(final_result)
}
