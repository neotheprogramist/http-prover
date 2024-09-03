use reqwest::Error as ReqwestError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProveErrors {
    #[error(transparent)]
    SdkErrors(#[from] prover_sdk::errors::SdkErrors),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Prover response error: {0}")]
    ProveResponseError(String),
    #[error("Missing program input")]
    MissingProgramInput,
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    RequestFailed(#[from] ReqwestError),
    #[error("{0}")]
    Custom(String),
}
