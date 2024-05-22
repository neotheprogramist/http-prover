use clap::{arg, Parser};
use sdk::{load_cairo0, load_cairo1, ProverAccessKey, ProverSDK};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::io::Read;
use tokio::io::{self as async_io, AsyncReadExt};
use url::Url;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[clap(author, version, about, long_about = None)]
pub struct CliInput {
    #[arg(short, long)]
    pub key: String,
    #[arg(short, long, default_value_t = 1)]
    pub cairo_version: usize, // 0 or 1,
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let args = CliInput::parse();
    let prover_url = Url::parse("http://localhost:3000").unwrap();
    let secret_key = ProverAccessKey::from_hex_string(&args.key).unwrap();
    let sdk = ProverSDK::new(secret_key, prover_url).await.unwrap();
    // Assume the input data is in JSON format as required by the SDK
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    println!("Received input: {}", input.trim());
    let proof = match args.cairo_version {
        0 => {
            let data = load_cairo0((&input).into()).await.unwrap();
            sdk.prove_cairo0(data).await.unwrap()
        }
        1 => {
            let data = load_cairo1((&input).into()).await.unwrap();
            sdk.prove_cairo1(data).await.unwrap()
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
