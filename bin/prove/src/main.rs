use clap::{arg, Parser};
use sdk::{Cairo0ProverInput, Cairo1ProverInput, ProverAccessKey, ProverSDK};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::io::Read;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ProveError {
    #[error("Failed to read: {0}")]
    Read(#[from] io::Error),

    #[error("Failed to parse prover key")]
    DecodeKey(prefix_hex::Error),

    #[error("Failed to initialize or authenticate prover SDK")]
    Initialize(sdk::ProverSdkErrors),

    #[error("Failed to parse input: {0}")]
    ParseInput(#[from] serde_json::Error),

    #[error("Failed to prove: {0}")]
    Prove(sdk::ProverSdkErrors),
}

#[derive(Parser, Debug, Serialize, Deserialize)]
#[clap(author, version, about, long_about = None)]
pub struct CliInput {
    #[arg(short = 'k', long)]
    pub prover_key: String,

    #[arg(short, long, default_value_t = 1)]
    pub cairo_version: usize, // 0 or 1,

    #[arg(short, long, default_value = "http://localhost:3000")]
    pub prover_url: Url,
}

#[tokio::main]
async fn main() -> Result<(), ProveError> {
    let args = CliInput::parse();
    let secret_key =
        ProverAccessKey::from_hex_string(&args.prover_key).map_err(ProveError::DecodeKey)?;
    let sdk = ProverSDK::new(secret_key, args.prover_url)
        .await
        .map_err(ProveError::Initialize)?;

    // Assume the input data is in JSON format as required by the SDK
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let proof = match args.cairo_version {
        0 => {
            let input: Cairo0ProverInput =
                serde_json::from_str(&input).map_err(ProveError::ParseInput)?;
            sdk.prove_cairo0(input).await.map_err(ProveError::Prove)?
        }
        1 => {
            let input: Cairo1ProverInput =
                serde_json::from_str(&input).map_err(ProveError::ParseInput)?;
            sdk.prove_cairo1(input).await.map_err(ProveError::Prove)?
        }
        _ => panic!("Invalid cairo version"),
    };

    let proof_json: serde_json::Value =
        serde_json::from_str(&proof).expect("Failed to parse result");

    println!(
        "{}",
        serde_json::to_string_pretty(&proof_json).expect("Failed to serialize result")
    );

    Ok(())
}
