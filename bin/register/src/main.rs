use clap::{arg, Parser};
use ed25519_dalek::VerifyingKey;
use prover_sdk::{access_key::ProverAccessKey, errors::SdkErrors, sdk::ProverSDK};
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
async fn main() -> Result<(), SdkErrors> {
    let args = Args::parse();

    let key =
        ProverAccessKey::from_hex_string(&args.private_key).map_err(|_| SdkErrors::InvalidKey)?;

    let mut sdk = ProverSDK::new(args.url, key)
        .await
        .expect("Failed to create SDK instance");

    let bytes: [u8; 32] =
        prefix_hex::decode(&args.added_key).map_err(|e| SdkErrors::PrefixError(e.to_string()))?;
    let added_key = VerifyingKey::from_bytes(&bytes).map_err(|_| SdkErrors::InvalidKey)?;

    sdk.register(added_key).await
}
