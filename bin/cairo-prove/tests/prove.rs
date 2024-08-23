use crate::common::read_file;
use cairo_proof_parser::output::{extract_output, ExtractOutputResult};
use cairo_prove::{prove, CliInput};
use serde_json::Value;
use std::path::PathBuf;
use url::Url;
mod common;

#[tokio::test]
async fn test_cairo1_fibonacci() -> Result<(), cairo_prove::ProveError> {
    let key = "0x5883b0e30b008e48af3d0bf5cfc138fb6093496da6f87d24b65def88470356d3";
    let args = CliInput {
        key: key.to_string(),
        cairo_version: 1,
        url: Url::parse("http://localhost:3040").unwrap(),
    };

    let prover_input =
        read_file(PathBuf::from("examples/Cairo/fibonacci_prover_input.json")).await?;
    let proof = prove(args, prover_input).await?;
    assert!(extract_output(&proof).is_ok());
    Ok(())
}
#[tokio::test]
async fn test_cairo0_fibonacci() -> Result<(), cairo_prove::ProveError> {
    let key = "0x5883b0e30b008e48af3d0bf5cfc138fb6093496da6f87d24b65def88470356d3";
    let args = CliInput {
        key: key.to_string(),
        cairo_version: 0,
        url: Url::parse("http://localhost:3040").unwrap(),
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
    Ok(())
}
