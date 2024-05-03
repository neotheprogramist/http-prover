use reqwest::Error as ReqwestError;
use thiserror::Error;
use tokio::fs::File;
use tokio::time::Duration;
mod models;
mod prover_sdk;
use reqwest::Url;
use url:: ParseError;

mod models;
mod prover_sdk;


#[derive(Debug,Error)]
enum ProverSdkErrors {
    #[error("HTTP request failed")]
    RequestFailed(#[from] ReqwestError),

    #[error("JSON parsing failed")]
    JsonParsingFailed,

    #[error("Failed to serialize")]
    SerdeError(#[from] serde_json::Error),

    #[error("Nonce not found in the response")]
    NonceNotFound,

    #[error("JWT token not found in the response")]
    JwtTokenNotFound,

    #[error("Reading input file failed")]
    ReadFileError(#[from] std::io::Error),

    #[error("Expiration date not found")]
    ExpirationNotFound,

    #[error("Signing key not found")]
    SigningKeyNotFound,
    #[error("Failed to parse to url")]
    UrlParseError(#[from] ParseError),
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
