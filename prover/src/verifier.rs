use crate::{
    auth::jwt::Claims, errors::ProverError, extractors::workdir::TempDirHandle, server::AppState,
    utils::job::JobStore,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::models::JobStatus;
use serde_json::json;
use std::{process::Command, sync::Arc};
use tempfile::TempDir;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

pub async fn root(
    State(app_state): State<AppState>,
    TempDirHandle(dir): TempDirHandle,
    _claims: Claims,
    Json(proof): Json<String>,
) -> impl IntoResponse {
    let job_store = app_state.job_store.clone();
    let job_id = job_store.create_job().await;

    tokio::spawn({
        async move {
            if let Err(e) =
                verify_proof(job_id, job_store.clone(), dir, proof, app_state.sse_tx).await
            {
                job_store
                    .update_job_status(job_id, JobStatus::Failed, Some(e.to_string()))
                    .await;
            }
        }
    });

    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}

pub async fn verify_proof(
    job_id: u64,
    job_store: JobStore,
    dir: TempDir,
    proof: String,
    sender: Arc<Mutex<Sender<String>>>,
) -> Result<(), ProverError> {
    job_store
        .update_job_status(job_id, JobStatus::Running, None)
        .await;

    // Define the path for the proof file
    let path = dir.into_path();
    let file = path.join("proof");

    // Write the proof string to the file
    std::fs::write(&file, &proof)?;

    // Create the command to run the verifier
    let mut command = Command::new("cpu_air_verifier");
    command.arg("--in_file").arg(&file);

    // Execute the command and capture the status
    let status = command.status()?;
    // Remove the proof file
    std::fs::remove_file(&file)?;
    // Check if the command was successful

    job_store
        .update_job_status(
            job_id,
            JobStatus::Completed,
            Some(status.success().to_string()),
        )
        .await;
    let sender = sender.lock().await;
    if sender.receiver_count() > 0 {
        sender
            .send(serde_json::to_string(&(JobStatus::Completed, job_id))?)
            .map_err(|e| ProverError::SseError(e.to_string()))?;
    }
    Ok(())
}
