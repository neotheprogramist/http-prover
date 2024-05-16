use clap::{arg, Parser};
use ed25519_dalek::VerifyingKey;
use sdk::{ProverAccessKey, ProverSDK};
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
    pub register_url: Url,

    #[arg(long, short, env)]
    pub auth_url: Url,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let key = ProverAccessKey::from_hex_string(&args.private_key).unwrap();

    let mut sdk = ProverSDK::new(key, args.auth_url.clone(), args.auth_url)
        .await
        .expect("Failed to create SDK instance");

    let bytes = hex::decode(&args.added_key).unwrap();
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    let added_key = VerifyingKey::from_bytes(&array).unwrap();

    sdk.register(added_key, args.register_url).await.unwrap();
}
