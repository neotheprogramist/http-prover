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
    let program_input_path: PathBuf = Path::new("resources/CairoZero/input.json").to_path_buf();
    let program_path: PathBuf = Path::new("resources/CairoZero/program.json").to_path_buf();
    let proof_path: PathBuf = Path::new("program_proof_cairo0.json").to_path_buf();

    let input = serde_json::to_string(&program_input.program_input)?;
    let program = serde_json::to_string(&program_input.program)?;
    let layout = program_input.layout;

    fs::write(&program_input_path, input.clone()).await?;
    fs::write(&program_path, program.clone()).await?;

    //run cairo-run
    let mut command = Command::new("cairo-run");
    command
        .arg("--trace_file=resources/CairoZero/program_trace.trace")
        .arg("--memory_file=resources/CairoZero/program_memory.memory")
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input=resources/CairoZero/program_public_input.json")
        .arg("--air_private_input=resources/CairoZero/program_private_input.json")
        .arg("--program_input")
        .arg(program_input_path)
        .arg("--program")
        .arg(program_path);

    let mut child = command.spawn()?;
    let _status = child.wait()?;

    generate_config::generate(
        "resources/CairoZero/program_public_input.json",
        "resources/CairoZero/cpu_air_params.json",
    );

    //run cpu_air_prover
    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--public_input_file=resources/CairoZero/program_public_input.json")
        .arg("--private_input_file=resources/CairoZero/program_private_input.json")
        .arg("--prover_config_file=examples/CairoZero/cpu_air_prover_config.json")
        .arg("--parameter_file=resources/CairoZero/cpu_air_params.json")
        .arg("-generate_annotations")
        .arg("--out_file")
        .arg(proof_path.clone());

    let mut child_proof = command_proof.spawn()?;
    let _status_proof = child_proof.wait()?;

    let result = fs::read_to_string(proof_path).await?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;

    Ok(final_result)
}
