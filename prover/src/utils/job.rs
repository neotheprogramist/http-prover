use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use common::models::JobStatus;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::{auth::jwt::Claims, server::AppState};

#[derive(Clone)]
pub struct Job {
    pub id: u64,
    pub status: JobStatus,
    pub result: Option<String>,
    pub created: Instant,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JobResponse {
    InProgress { id: u64, status: JobStatus },
    Completed { result: String, status: JobStatus },
    Failed { error: String },
}

#[derive(Default, Clone)]
pub struct JobStore {
    inner: Arc<Mutex<JobStoreInner>>,
}

impl JobStore {
    pub async fn create_job(&self) -> u64 {
        self.inner.lock().await.create_job()
    }
    pub async fn update_job_status(&self, job_id: u64, status: JobStatus, result: Option<String>) {
        self.inner
            .lock()
            .await
            .update_job_status(job_id, status, result)
    }
    pub async fn get_job(&self, id: u64) -> Option<Job> {
        self.inner.lock().await.get_job(id)
    }
}

#[derive(Default)]
struct JobStoreInner {
    jobs: BTreeMap<u64, Job>,
    counter: u64,
}

impl JobStoreInner {
    pub fn create_job(&mut self) -> u64 {
        let job_id = self.counter;
        self.counter += 1;
        let new_job = Job {
            id: job_id,
            status: JobStatus::Pending,
            result: None,
            created: Instant::now(),
        };
        self.jobs.insert(job_id, new_job);
        self.clear_old_jobs();
        job_id
    }
    pub fn update_job_status(&mut self, job_id: u64, status: JobStatus, result: Option<String>) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = status;
            job.result = result;
        }
        self.clear_old_jobs()
    }
    pub fn get_job(&mut self, id: u64) -> Option<Job> {
        let job = self.jobs.get(&id).cloned();
        self.clear_old_jobs();
        job
    }
    // Clear old jobs so that the memory doesn't go balistic if the server runs for a long time
    fn clear_old_jobs(&mut self) {
        let expiry_duration = Duration::from_secs(5 * 60 * 60); // 5 hours
        while let Some((id, job)) = self.jobs.pop_first() {
            if job.created.elapsed() < expiry_duration {
                self.jobs.insert(id, job);
                break;
            }
        }
    }
}

pub async fn get_job(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    _claims: Claims,
) -> impl IntoResponse {
    if let Some(job) = app_state.job_store.get_job(id).await {
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
