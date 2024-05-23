use crate::common::{read_file, spawn_prover};
use cairo_proof_parser::output::{extract_output, ExtractOutputResult};
use prove::{prove, CliInput};
use serde_json::Value;
use std::path::PathBuf;

mod common;

#[tokio::test]
async fn test_cairo1_fibonacci() -> Result<(), prove::ProveError> {
    let (handle, key, url) = spawn_prover().await;

    let args = CliInput {
        prover_key: key.signing_key_as_hex_string(),
        cairo_version: 1,
        prover_url: url,
    };

    let prover_input = read_file(PathBuf::from("examples/Cairo/prover_input.json")).await?;
    let proof = prove(args, prover_input).await?;
    assert!(extract_output(&proof).is_ok());
    handle.abort();
    Ok(())
}
#[tokio::test]
async fn test_cairo0_fibonacci() -> Result<(), prove::ProveError> {
    let (handle, key, url) = spawn_prover().await;

    let args = CliInput {
        prover_key: key.signing_key_as_hex_string(),
        cairo_version: 0,
        prover_url: url,
    };
    let prover_input = read_file(PathBuf::from("examples/CairoZero/prover_input.json")).await?;
    let program_input: Value = serde_json::from_str(&prover_input)?;

    let fibonacci_claim_index = program_input
        .get("program_input")
        .and_then(|v| v.get("fibonacci_claim_index"))
        .and_then(|v| v.as_u64())
        .unwrap();

    let proof = prove(args, prover_input).await?;

    let ExtractOutputResult { program_output, .. } = extract_output(&proof).unwrap();
    let expected_input: u64 = program_output[0].try_into().unwrap();

    assert_eq!(
        expected_input, fibonacci_claim_index,
        "Fibonacci index mismatch."
    );

    handle.abort();

    Ok(())
}
