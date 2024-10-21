pub mod auth;
pub mod cairo1_run;
pub mod errors;
pub mod extractors;
pub mod prove;
pub mod server;
pub mod sse;
pub mod threadpool;
pub mod utils;
pub mod verifier;
use clap::{arg, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env, default_value = "0.0.0.0")]
    pub host: String,
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,
    #[arg(long, short, env, default_value = "3600")]
    pub message_expiration_time: usize,
    #[arg(long, short, env, default_value = "3600")]
    pub session_expiration_time: usize,
    #[arg(long, short, env)]
    pub jwt_secret_key: String,
    #[arg(long, env, default_value = "authorized_keys.json")]
    pub authorized_keys_path: PathBuf,
    #[arg(long, env, value_delimiter = ',')]
    pub authorized_keys: Vec<String>,
    #[arg(long, env, default_value = "4")]
    pub num_workers: usize,
    #[arg(long, env, value_delimiter = ',')]
    pub admin_keys: Vec<String>,
}
