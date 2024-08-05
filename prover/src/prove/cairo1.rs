use crate::auth::jwt::Claims;
use crate::prove::errors::ProveError;
use axum::Json;
use common::Cairo1ProverInput;
use config_generator::generate_config::generate;
use serde_json::Value;
use tokio::fs;

use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn root(
    _claims: Claims,
    Json(program_input): Json<Cairo1ProverInput>,
) -> Result<String, ProveError> {
    let program_input_path: PathBuf = Path::new("resources/Cairo/input.txt").to_path_buf();
    let program_path: PathBuf = Path::new("resources/Cairo/program.json").to_path_buf();
    let proof_path: PathBuf = Path::new("program_proof_cairo1.json").to_path_buf();

    let input = serde_json::to_string(&program_input.program_input)?;
    let program = serde_json::to_string(&program_input.program)?;
    let layout = program_input.layout;

    let json_value: Value = serde_json::from_str(&input)?;

    // Get the array from the JSON
    if let Value::Array(array) = json_value {
        // Convert each JSON value to a String and join them with a space
        let output_str = array
            .into_iter()
            .filter_map(|v| {
                // Convert the value to a String if it's a string
                v.as_str().map(|s| s.to_owned())
            })
            .collect::<Vec<String>>()
            .join(" ");

        // Write the final output to a file
        fs::write(program_input_path.clone(), output_str).await?;
    } else {
        eprintln!("Expected a JSON array");
        std::process::exit(1);
    }

    // // Write `program` field to a file
    fs::write(&program_path, program.clone()).await?;
    //run cairo1-run
    let mut command = Command::new("cairo1-run");
    command
        .arg("--trace_file=resources/Cairo/program_trace.trace")
        .arg("--memory_file=resources/Cairo/program_memory.memory")
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input=resources/Cairo/program_public_input.json")
        .arg("--air_private_input=resources/Cairo/program_private_input.json")
        .arg("--args_file")
        .arg(program_input_path)
        .arg(program_path.clone());

    // Start the process
    let mut child = command.spawn()?;

    // Wait for the process to finish
    let _status = child.wait()?;

    generate(
        "resources/Cairo/program_public_input.json",
        "resources/Cairo/cpu_air_params.json",
    );

    //run cpu_air_prover
    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--out_file")
        .arg(proof_path.clone())
        .arg("--private_input_file=resources/Cairo/program_private_input.json")
        .arg("--public_input_file=resources/Cairo/program_public_input.json")
        .arg("--prover_config_file=examples/Cairo/cpu_air_prover_config.json")
        .arg("--parameter_file=resources/Cairo/cpu_air_params.json")
        .arg("-generate-annotations");

    let mut child_proof = command_proof.spawn()?;

    // Wait for the process to finish
    let _status_proof = child_proof.wait()?;

    let result = fs::read_to_string(proof_path).await?;

    // Deserialize the string into a serde_json::Value
    let proof: Value = serde_json::from_str(&result)?;

    let final_result = serde_json::to_string_pretty(&proof)?;

    Ok(final_result)
}
