use super::run::RunPaths;
use super::CairoVersionedInput;
use crate::errors::ProverError;
use crate::utils::{config::Template, job::JobStore};
use common::models::JobStatus;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

pub async fn prove(
    job_id: u64,
    job_store: JobStore,
    dir: TempDir,
    program_input: CairoVersionedInput,
    sse_tx: Arc<Mutex<Sender<String>>>,
    n_queries: Option<u32>,
    pow_bits: Option<u32>,
) -> Result<(), ProverError> {
    job_store
        .update_job_status(job_id, JobStatus::Running, None)
        .await;

    let paths = ProvePaths::new(dir);

    program_input
        .prepare_and_run(&RunPaths::from(&paths))
        .await?;
    Template::generate_from_public_input_file(&paths.public_input_file, n_queries, pow_bits)?
        .save_to_file(&paths.params_file)?;

    let prove_status = paths.prove_command().spawn()?.wait().await?;
    let result = fs::read_to_string(&paths.proof_path)?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    let sender = sse_tx.lock().await;

    if prove_status.success() {
        job_store
            .update_job_status(job_id, JobStatus::Completed, Some(final_result))
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Completed, job_id))?)
                .unwrap();
        }
    } else {
        job_store
            .update_job_status(job_id, JobStatus::Failed, Some(final_result))
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Failed, job_id))?)
                .unwrap();
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(super) struct ProvePaths {
    pub(super) program_input: PathBuf,
    pub(super) program: PathBuf,
    pub(super) proof_path: PathBuf,
    pub(super) trace_file: PathBuf,
    pub(super) memory_file: PathBuf,
    pub(super) public_input_file: PathBuf,
    pub(super) private_input_file: PathBuf,
    pub(super) params_file: PathBuf,
    pub(super) config_file: PathBuf,
}

impl ProvePaths {
    pub fn new(base_dir: TempDir) -> Self {
        let path = base_dir.into_path();
        Self {
            program_input: path.join("program_input.json"),
            program: path.join("program.json"),
            proof_path: path.join("program_proof_cairo.json"),
            trace_file: path.join("program_trace.trace"),
            memory_file: path.join("program_memory.memory"),
            public_input_file: path.join("program_public_input.json"),
            private_input_file: path.join("program_private_input.json"),
            params_file: path.join("cpu_air_params.json"),
            config_file: PathBuf::from_str("config/cpu_air_prover_config.json").unwrap(),
        }
    }
    pub fn prove_command(&self) -> Command {
        let mut command = Command::new("cpu_air_prover");
        command
            .arg("--out_file")
            .arg(&self.proof_path)
            .arg("--private_input_file")
            .arg(&self.private_input_file)
            .arg("--public_input_file")
            .arg(&self.public_input_file)
            .arg("--prover_config_file")
            .arg(&self.config_file)
            .arg("--parameter_file")
            .arg(&self.params_file)
            .arg("-generate-annotations");
        command
    }
}
