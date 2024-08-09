use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use axum::Json;
use common::Cairo1ProverInput;
use config_generator::generate_config;
use serde_json::Value;
use tokio::fs;

use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn root(
    _claims: Claims,
    Json(program_input): Json<Cairo1ProverInput>,
) -> Result<String, ProveError> {
    let program_input_path: PathBuf = Path::new("resources/cairo/input.txt").to_path_buf();
    let program_path: PathBuf = Path::new("resources/cairo/program.json").to_path_buf();
    let proof_path: PathBuf = Path::new("program_proof_cairo1.json").to_path_buf();
    let trace_file = Path::new("resources/cairo/program_trace.trace").to_path_buf();
    let memory_file = Path::new("resources/cairo/program_memory.memory").to_path_buf();
    let public_input_file = Path::new("resources/cairo/program_public_input.json").to_path_buf();
    let private_input_file = Path::new("resources/cairo/program_private_input.json").to_path_buf();
    let params_file = Path::new("resources/cairo/cpu_air_params.json").to_path_buf();
    let config_file = Path::new("config/cpu_air_prover_config.json").to_path_buf();

    let input = serde_json::to_string(&program_input.program_input)?;
    let program = serde_json::to_string(&program_input.program)?;
    let layout = program_input.layout;

    let json_value: Value = serde_json::from_str(&input)?;

    if let Value::Array(array) = json_value {
        let output_str = array
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_owned()))
            .collect::<Vec<String>>()
            .join(" ");

        fs::write(program_input_path.clone(), output_str).await?;
    } else {
        eprintln!("Expected a JSON array");
        std::process::exit(1);
    }

    fs::write(&program_path, &program).await?;

    //run cairo1-run
    let mut command = Command::new("cairo1-run");
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
        .arg("--args_file")
        .arg(&program_input_path)
        .arg(&program_path);

    let mut child = command.spawn()?;
    let _status = child.wait()?;

    generate_config::generate(
        "resources/cairo/program_public_input.json",
        "resources/cairo/cpu_air_params.json",
    );

    //run cpu_air_prover
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
