use super::CairoVersionedInput;
use crate::errors::ProverError;
use crate::utils::{config::Template, job::JobStore};
use common::models::JobStatus;
use serde_json::Value;
use starknet_types_core::felt::Felt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tracing::trace;

pub async fn prove(
    job_id: u64,
    job_store: JobStore,
    dir: TempDir,
    program_input: CairoVersionedInput,
    sse_tx: Arc<Mutex<Sender<String>>>,
) -> Result<(), ProverError> {
    job_store
        .update_job_status(job_id, JobStatus::Running, None)
        .await;
    let path = dir.into_path();
    let program_input_path: PathBuf = path.join("program_input.json");
    let program_path: PathBuf = path.join("program.json");
    let proof_path: PathBuf = path.join("program_proof_cairo.json");
    let trace_file = path.join("program_trace.trace");
    let memory_file = path.join("program_memory.memory");
    let public_input_file = path.join("program_public_input.json");
    let private_input_file = path.join("program_private_input.json");
    let params_file = path.join("cpu_air_params.json");
    let config_file = PathBuf::from_str("config/cpu_air_prover_config.json")?;
    match program_input {
        CairoVersionedInput::Cairo(input) => {
            let program = serde_json::to_string(&input.program)?;
            let layout = input.layout;
            let input = prepare_input(&input.program_input);
            fs::write(&program_path, &program)?;
            fs::write(&program_input_path, &input)?;
            cairo_run(
                &trace_file,
                &memory_file,
                layout,
                &public_input_file,
                &private_input_file,
                &program_input_path,
                &program_path,
            )
            .await?;
        }
        CairoVersionedInput::Cairo0(input) => {
            fs::write(
                program_input_path.clone(),
                serde_json::to_string(&input.program_input)?,
            )?;
            fs::write(&program_path, serde_json::to_string(&input.program)?)?;
            let layout = input.layout;
            cairo0_run(
                &trace_file,
                &memory_file,
                layout,
                &public_input_file,
                &private_input_file,
                &program_input_path,
                &program_path,
            )
            .await?;
        }
    }

    Template::generate_from_public_input_file(&public_input_file)?.save_to_file(&params_file)?;

    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--out_file")
        .arg(&proof_path)
        .arg("--private_input_file")
        .arg(&private_input_file)
        .arg("--public_input_file")
        .arg(&public_input_file)
        .arg("--prover_config_file")
        .arg(&config_file)
        .arg("--parameter_file")
        .arg(&params_file)
        .arg("-generate-annotations");

    let mut child_proof = command_proof.spawn()?;
    let status_proof = child_proof.wait().await?;
    let result = fs::read_to_string(&proof_path)?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    let sender = sse_tx.lock().await;

    if status_proof.success() {
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

pub async fn cairo0_run(
    trace_file: &PathBuf,
    memory_file: &PathBuf,
    layout: String,
    public_input_file: &PathBuf,
    private_input_file: &PathBuf,
    program_input_path: &PathBuf,
    program_path: &PathBuf,
) -> Result<(), ProverError> {
    trace!("Running cairo0-run");
    let mut command = Command::new("cairo-run");
    command
        .arg("--trace_file")
        .arg(trace_file)
        .arg("--memory_file")
        .arg(memory_file)
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input")
        .arg(public_input_file)
        .arg("--air_private_input")
        .arg(private_input_file)
        .arg("--program_input")
        .arg(program_input_path)
        .arg("--program")
        .arg(program_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = command.spawn()?;
    let output = child.wait_with_output().await?;

    // Capture stderr in case of an error
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ProverError::CustomError(stderr.into()));
    }
    Ok(())
}
pub async fn cairo_run(
    trace_file: &PathBuf,
    memory_file: &PathBuf,
    layout: String,
    public_input_file: &PathBuf,
    private_input_file: &PathBuf,
    program_input_path: &PathBuf,
    program_path: &PathBuf,
) -> Result<(), ProverError> {
    let mut command = Command::new("cairo1-run");
    command
        .arg("--trace_file")
        .arg(trace_file)
        .arg("--memory_file")
        .arg(memory_file)
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input")
        .arg(public_input_file)
        .arg("--air_private_input")
        .arg(private_input_file)
        .arg("--args_file")
        .arg(program_input_path)
        .arg(program_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = command.spawn()?;
    let output = child.wait_with_output().await?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ProverError::CustomError(stderr.into()))
    }
}

pub fn prepare_input(felts: &[Felt]) -> String {
    felts
        .iter()
        .fold("[".to_string(), |a, i| a + &i.to_string() + " ")
        .trim_end()
        .to_string()
        + "]"
}

#[test]
fn test_prepare_input() {
    assert_eq!("[]", prepare_input(&[]));
    assert_eq!("[1]", prepare_input(&[1.into()]));
    assert_eq!(
        "[1 2 3 4]",
        prepare_input(&[1.into(), 2.into(), 3.into(), 4.into()])
    );
}
