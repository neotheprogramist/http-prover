use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use axum::Json;
use common::Cairo0ProverInput;
use config_generator::generate_config;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio::fs;

pub async fn root(
    _claims: Claims,
    Json(program_input): Json<Cairo0ProverInput>,
) -> Result<String, ProveError> {
    let program_input_path: PathBuf = Path::new("resources/cairoZero/input.json").to_path_buf();
    let program_path: PathBuf = Path::new("resources/cairoZero/program.json").to_path_buf();
    let proof_path: PathBuf = Path::new("program_proof_cairo0.json").to_path_buf();
    let trace_file = Path::new("resources/cairoZero/program_trace.trace").to_path_buf();
    let memory_file = Path::new("resources/cairoZero/program_memory.memory").to_path_buf();
    let public_input_file =
        Path::new("resources/cairoZero/program_public_input.json").to_path_buf();
    let private_input_file =
        Path::new("resources/cairoZero/program_private_input.json").to_path_buf();
    let params_file = Path::new("resources/cairoZero/cpu_air_params.json").to_path_buf();
    let config_file = Path::new("config/cpu_air_prover_config.json").to_path_buf();

    let input = serde_json::to_string(&program_input.program_input)?;
    let program = serde_json::to_string(&program_input.program)?;
    let layout = program_input.layout;

    fs::write(&program_input_path, input.clone()).await?;
    fs::write(&program_path, program.clone()).await?;

    //run cairo-run
    let mut command = Command::new("cairo-run");
    command
        .arg("--trace_file")
        .arg(&trace_file)
        .arg("--memory_file")
        .arg(&memory_file)
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input")
        .arg(&public_input_file)
        .arg("--air_private_input")
        .arg(&private_input_file)
        .arg("--program_input")
        .arg(&program_input_path)
        .arg("--program")
        .arg(&program_path);

    let mut child = command.spawn()?;
    let _status = child.wait()?;

    generate_config::generate(
        "resources/cairoZero/program_public_input.json",
        "resources/cairoZero/cpu_air_params.json",
    );

    //run cpu_air_prover
    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--public_input_file")
        .arg(&public_input_file)
        .arg("--private_input_file")
        .arg(&private_input_file)
        .arg("--prover_config_file")
        .arg(&config_file)
        .arg("--parameter_file")
        .arg(&params_file)
        .arg("-generate_annotations")
        .arg("--out_file")
        .arg(&proof_path);

    let mut child_proof = command_proof.spawn()?;
    let _status_proof = child_proof.wait()?;

    let result = fs::read_to_string(&proof_path).await?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;

    fs::remove_file(&program_input_path).await?;
    fs::remove_file(&program_path).await?;
    fs::remove_file(&proof_path).await?;
    fs::remove_file(&trace_file).await?;
    fs::remove_file(&memory_file).await?;
    fs::remove_file(&public_input_file).await?;
    fs::remove_file(&private_input_file).await?;
    fs::remove_file(&params_file).await?;

    Ok(final_result)
}
