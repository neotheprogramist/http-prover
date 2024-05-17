pub mod auth;
pub mod prove;
pub mod server;
use std::path::PathBuf;

use clap::{arg, Parser, ValueHint};

/// Command line arguments for the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Host address to bind the server
    #[arg(long, env, default_value = "0.0.0.0")]
    pub host: String,

    /// Port to listen on
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,
    #[arg(long, short, env)]
    pub jwt_secret_key: String,
    #[arg(long, short, env)]
    pub message_expiration_time: u32,
    #[arg(long, short, env)]
    pub session_expiration_time: u32,
    #[arg(long, short = 'k', env)]
    pub private_key: String,
    #[arg(long, short, env, value_hint = ValueHint::FilePath)]
    pub authorized_keys_path: Option<PathBuf>,
    #[arg(long, short = 'f', env)]
    pub authorized_keys: Option<Vec<String>>,
}
