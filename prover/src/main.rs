use clap::Parser;
use prover::prove::errors::ServerError;
use prover::server::start;
use prover::Args;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let args: Args = Args::parse();

    // Start the server with the specified address
    start(args).await?;

    Ok(())
}
