use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use axum::Json;
use common::Cairo0ProverInput;
use config_generator::generate_config::generate;
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
    let program_input_path: PathBuf = Path::new("examples/CairoZero/input.json").to_path_buf();
    let program_path: PathBuf = Path::new("examples/CairoZero/program.json").to_path_buf();
    let proof_path: PathBuf = Path::new("program_proof.json").to_path_buf();

    let input = serde_json::to_string(&program_input.program_input)?;
    let program = serde_json::to_string(&program_input.program)?;
    let layout = program_input.layout;

    // Write `program_input` field to a file
    fs::write(&program_input_path, input.clone()).await?;

    // // Write `program` field to a file
    fs::write(&program_path, program.clone()).await?;

    let mut command = Command::new("cairo-run");
    command
        .arg("--trace_file=examples/CairoZero/program_trace.trace")
        .arg("--memory_file=examples/CairoZero/program_memory.memory")
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input=examples/CairoZero/program_public_input.json")
        .arg("--air_private_input=examples/CairoZero/program_private_input.json")
        .arg("--program_input")
        .arg(program_input_path)
        .arg("--program")
        .arg(program_path);

    // Start the process
    let mut child = command.spawn()?;

    // Wait for the process to finish
    let _status = child.wait()?;

    //HERE CONFIG-GENERATOR should return cpu_air_prover_config.json
    generate(
        "examples/CairoZero/program_public_input.json",
        "examples/CairoZero/cpu_air_params.json",
    );

    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--public_input_file=examples/CairoZero/program_public_input.json")
        .arg("--private_input_file=examples/CairoZero/program_private_input.json")
        .arg("--prover_config_file=examples/CairoZero/cpu_air_prover_config.json")
        .arg("--parameter_file=examples/CairoZero/cpu_air_params.json")
        .arg("-generate_annotations")
        .arg("--out_file=program_proof.json");
    
    let mut child_proof = command_proof.spawn()?;

    // Wait for the process to finish
    let _status_proof = child_proof.wait()?;

    let result = fs::read_to_string(proof_path).await?;

    // Deserialize the string into a serde_json::Value
    let proof: Value = serde_json::from_str(&result)?;

    let final_result = serde_json::to_string_pretty(&proof)?;

    Ok(final_result)
}
