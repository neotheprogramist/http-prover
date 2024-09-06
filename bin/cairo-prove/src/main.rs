use cairo_prove::errors::ProveErrors;
use cairo_prove::prove::prove;
use cairo_prove::{
    fetch::{fetch_job_polling, fetch_job_sse},
    Args,
};
use clap::Parser;
use prover_sdk::access_key::ProverAccessKey;
use prover_sdk::sdk::ProverSDK;
#[tokio::main]
pub async fn main() -> Result<(), ProveErrors> {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let access_key = ProverAccessKey::from_hex_string(&args.prover_access_key.clone())?;
    let sdk = ProverSDK::new(args.prover_url.clone(), access_key).await?;
    let job = prove(args.clone(), sdk.clone()).await?;
    if args.wait {
        let job = if args.sse {
            fetch_job_sse(sdk, job).await?
        } else {
            fetch_job_polling(sdk, job).await?
        };
        let path: std::path::PathBuf = args.program_output;
        std::fs::write(path, job)?;
    }

    Ok(())
}
