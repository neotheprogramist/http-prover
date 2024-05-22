use clap::{arg, Parser};
use sdk::{load_cairo0, load_cairo1, ProverAccessKey, ProverSDK};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Write};
use url::Url;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[clap(author, version, about, long_about = None)]
pub struct CliInput {
    #[arg(short, long)]
    pub key: String,
    #[arg(short, long, default_value_t = 1)]
    pub cairo_version: usize, // 0 or 1,
    pub file: std::path::PathBuf,
}
async fn prove_to_json(proof: String) {
    let mut file = File::create("result.json").expect("Failed to create file");
    let proof_json: serde_json::Value =
        serde_json::from_str(&proof).expect("Failed to parse result");
    let serialized = serde_json::to_string_pretty(&proof_json).expect("Failed to serialize result");
    file.write_all(serialized.as_bytes())
        .expect("Failed to write to file");
}

#[tokio::main]
async fn main() {
    let args = CliInput::parse();
    let prover_url = Url::parse("http://localhost:3000").unwrap();
    let secret_key = ProverAccessKey::from_hex_string(&args.key).unwrap();
    let sdk = ProverSDK::new(secret_key, prover_url).await.unwrap();
    match args.cairo_version {
        0 => {
            let data = load_cairo0(args.file.clone()).await.unwrap();
            let proof = sdk.prove_cairo0(data).await.unwrap();
            prove_to_json(proof).await;
        }
        1 => {
            let data = load_cairo1(args.file.clone()).await.unwrap();
            let proof = sdk.prove_cairo1(data).await.unwrap();
            prove_to_json(proof).await;
        }
        _ => panic!("Invalid cairo version"),
    }
}
