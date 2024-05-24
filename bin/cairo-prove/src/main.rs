use clap::Parser;
use cairo_prove::{prove, CliInput, ProveError};
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
pub async fn main() -> Result<(), ProveError> {
    let args = CliInput::parse();

    // Assume the input data is in JSON format as required by the SDK
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).await?;

    println!("{}", prove(args, input).await?);
    Ok(())
}
