pub mod auth;
pub mod prove;
pub mod server;
use std::path::PathBuf;

use clap::{arg, Parser, ValueHint};
use url::Url;

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
    #[arg(long, short, env, value_hint = ValueHint::FilePath)]
    pub authorized_keys_path: Option<PathBuf>,
    #[arg(long, short = 'f', env)]
    pub authorized_keys: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AcmeArgs {
    #[arg(short = 'i', long, env)]
    pub domain_identifiers: Vec<String>,
    #[arg(short = 'm', long, env)]
    pub contact_mails: Vec<String>,
    #[arg(short = 't', long, env)]
    pub api_token: String,
    #[arg(short = 'z', long, env)]
    pub zone_id: String,
    #[arg(short = 'u', long, env)]
    pub url: Url,
    #[arg(short = 'r', long, env)]
    pub renewal_threshold: i32,
}

impl AcmeArgs {
    #[must_use]
    pub fn split_identifiers(&self) -> Vec<String> {
        self.domain_identifiers
            .iter()
            .flat_map(|s| s.split(',').map(str::trim).map(String::from))
            .collect()
    }
    #[must_use]
    pub fn split_contact_mail(&self) -> Vec<String> {
        self.contact_mails
            .iter()
            .flat_map(|s| s.split(',').map(str::trim).map(String::from))
            .collect()
    }
    #[must_use]
    pub fn new() -> Self {
        let tmp = AcmeArgs::parse();
        let processed_identifiers = tmp.split_identifiers();
        let processed_contact_mail = tmp.split_contact_mail();
        AcmeArgs {
            domain_identifiers: processed_identifiers,
            contact_mails: processed_contact_mail,
            ..tmp
        }
    }
    pub fn domain_identifiers(&self) -> Vec<&str> {
        self.domain_identifiers
            .iter()
            .map(String::as_str)
            .clone()
            .collect()
    }
}
impl Default for AcmeArgs{
    fn default() -> Self{
        Self::new()
    }
}