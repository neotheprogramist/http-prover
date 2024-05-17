use clap::{arg, Parser};
use ed25519_dalek::VerifyingKey;
use prover_sdk::{ProverAccessKey, ProverSDK};
use url::Url;

/// Command line arguments for the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, env)]
    pub private_key: String,

    #[arg(long, short = 'k', env)]
    pub added_key: String,

    #[arg(long, short, env)]
    pub url: Url,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let key = ProverAccessKey::from_hex_string(&args.private_key).unwrap();

    let mut sdk = ProverSDK::new(key, args.url)
        .await
        .expect("Failed to create SDK instance");

    let bytes: [u8; 32] = prefix_hex::decode(&args.added_key).unwrap();
    let added_key = VerifyingKey::from_bytes(&bytes).unwrap();

    sdk.register(added_key).await.unwrap();
}
