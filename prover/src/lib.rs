pub mod auth;
pub mod prove;
pub mod server;
use clap::Parser;
/// Command line arguments for the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Host address to bind the server
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value = "3000")]
    port: u16,
    #[arg(long, env)]
    jwt_secret_key: String,
    #[arg(long, env)]
    message_expiration_time: u32,
    #[arg(long, env)]
    session_expiration_time: u32,
    #[arg(long, env)]
    private_key: String,
}
pub trait ProverInput {
    fn serialize(self) -> serde_json::Value;
}
