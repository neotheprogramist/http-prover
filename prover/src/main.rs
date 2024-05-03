use clap::Parser;
use server::{start, ServerError};

mod auth;
mod prove;
mod server;

/// Command line arguments for the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Host address to bind the server
    #[clap(long, default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[clap(long, default_value = "7003")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let args: Args = Args::parse();

    // Start the server with the specified address
    start(&args).await?;

    Ok(())
}
