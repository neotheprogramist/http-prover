use common::prover_input::*;
use helpers::fetch_job;
use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
use serde_json::Value;

use starknet_types_core::felt::Felt;

use url::Url;
mod helpers;

#[tokio::test]
async fn test_cairo_prove() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
    let program: CairoCompiledProgram = serde_json::from_str(&program).unwrap();
    let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
    let mut program_input: Vec<Felt> = Vec::new();
    for part in program_input_string.split(',') {
        let felt = Felt::from_dec_str(part).unwrap();
        program_input.push(felt);
    }
    let layout = "recursive".to_string();
    let data = CairoProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
    };
    let job = sdk.prove_cairo(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    let result = sdk.clone().verify(result.proof).await;
    assert!(result.is_ok(), "Failed to verify proof");
    assert_eq!("true", result.unwrap());
}

#[tokio::test]
async fn test_cairo0_prove() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let program = std::fs::read_to_string("../examples/cairo0/fibonacci_compiled.json").unwrap();
    let program: Cairo0CompiledProgram = serde_json::from_str(&program).unwrap();
    let program_input_string = std::fs::read_to_string("../examples/cairo0/input.json").unwrap();
    let program_input: Value = serde_json::from_str(&program_input_string).unwrap();
    let layout = "recursive".to_string();
    let data = Cairo0ProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
    };
    let job = sdk.prove_cairo0(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
}
#[tokio::test]
async fn test_cairo_multi_prove() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
    let program: CairoCompiledProgram = serde_json::from_str(&program).unwrap();
    let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
    let mut program_input: Vec<Felt> = Vec::new();
    for part in program_input_string.split(',') {
        let felt = Felt::from_dec_str(part).unwrap();
        program_input.push(felt);
    }
    let layout = "recursive".to_string();
    let data = CairoProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
    };
    let job1 = sdk.prove_cairo(data.clone()).await.unwrap();
    let job2 = sdk.prove_cairo(data.clone()).await.unwrap();
    let job3 = sdk.prove_cairo(data.clone()).await.unwrap();
    let result = fetch_job(sdk.clone(), job1).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
    let result = fetch_job(sdk.clone(), job2).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
    let result = fetch_job(sdk.clone(), job3).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
}
