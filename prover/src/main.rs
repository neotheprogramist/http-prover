use clap::Parser;
use prover::{errors::ProverError, server::start, Args};
#[tokio::main]
async fn main() -> Result<(), ProverError> {
    let args = Args::parse();
    start(args).await?;
    Ok(())
}
