use crate::auth::jwt::Claims;
use crate::extractors::workdir::TempDirHandle;
use crate::server::AppState;
use crate::threadpool::CairoVersionedInput;
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::cairo0_prover_input::Cairo0ProverInput;
use serde_json::json;

pub async fn root(
    State(app_state): State<AppState>,
    TempDirHandle(path): TempDirHandle,
    _claims: Claims,
    Json(program_input): Json<Cairo0ProverInput>,
) -> impl IntoResponse {
    let thread_pool = app_state.thread_pool.clone();
    let job_store = app_state.job_store.clone();
    let job_id = job_store.create_job().await;
    let thread = thread_pool.lock().await;
    thread
        .execute(
            job_id,
            job_store,
            path,
            CairoVersionedInput::Cairo0(program_input),
            app_state.sse_tx.clone(),
        )
        .await
        .into_response();
    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}
