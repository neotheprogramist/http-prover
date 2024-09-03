use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use common::models::JobStatus;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{auth::jwt::Claims, server::AppState};

#[derive(Serialize, Clone)]
pub struct Job {
    pub id: u64,
    pub status: JobStatus,
    pub result: Option<String>, // You can change this to any type based on your use case
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JobResponse {
    InProgress { id: u64, status: JobStatus },
    Completed { result: String, status: JobStatus },
    Failed { error: String },
}
pub type JobStore = Arc<Mutex<Vec<Job>>>;

pub async fn create_job(job_store: &JobStore) -> u64 {
    let mut jobs = job_store.lock().await;
    let job_id = jobs.len() as u64;
    let new_job = Job {
        id: job_id,
        status: JobStatus::Pending,
        result: None,
    };
    jobs.push(new_job);
    drop(jobs);
    job_id
}

pub async fn update_job_status(
    job_id: u64,
    job_store: &JobStore,
    status: JobStatus,
    result: Option<String>,
) {
    let mut jobs = job_store.lock().await;
    if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
        job.status = status;
        job.result = result;
    }
    drop(jobs);
}
pub async fn get_job(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    _claims: Claims,
) -> impl IntoResponse {
    let job_store = &app_state.job_store;
    let jobs = job_store.lock().await;
    if let Some(job) = jobs.iter().find(|job| job.id == id) {
        let (status, response) = match job.status {
            JobStatus::Pending | JobStatus::Running => (
                StatusCode::OK,
                Json(JobResponse::InProgress {
                    id: job.id,
                    status: job.status.clone(),
                }),
            ),
            JobStatus::Completed => (
                StatusCode::OK,
                Json(JobResponse::Completed {
                    status: job.status.clone(),
                    result: job
                        .result
                        .clone()
                        .unwrap_or_else(|| "No result available".to_string()),
                }),
            ),
            JobStatus::Failed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JobResponse::Failed {
                    error: job
                        .result
                        .clone()
                        .unwrap_or_else(|| "Unknown error".to_string()),
                }),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JobResponse::Failed {
                    error: "Unknown error".to_string(),
                }),
            ),
        };
        (status, response).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(format!("Job with id {} not found", id)),
        )
            .into_response()
    }
}
