pub mod auth;
pub mod prove;
pub mod server;
use clap::Parser;
/// Command line arguments for the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Host address to bind the server
    #[clap(long, default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[clap(long, default_value = "3000")]
    port: u16,
}