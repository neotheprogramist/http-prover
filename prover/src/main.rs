use clap::Parser;
use prover::prove::errors::ServerError;
use prover::server::start;
use prover::{AcmeArgs, Args};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let args: Args = Args::parse();
    let acme_args:AcmeArgs = AcmeArgs::new();
    // Start the server with the specified address
    start(args,acme_args).await?;

    Ok(())
}
