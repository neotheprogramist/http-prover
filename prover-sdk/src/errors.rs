use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkErrors {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Prover response error: {0}")]
    ProveResponseError(String),
    #[error("Get Job response error: {0}")]
    GetJobResponseError(String),
    #[error("Prefix error: {0}")]
    PrefixError(String),
    #[error("Nonce Request error: {0}")]
    NonceRequestFailed(String),
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error("Nonce not found")]
    NonceNotFound,
    #[error("Validate Signature response error: {0}")]
    ValidateSignatureResponseError(String),
    #[error("JWT Token not found")]
    JWTTokenNotFound,
    #[error("JWT Expiration not found")]
    JWTExpirationNotFound,
    #[error("Signing key not found")]
    SigningKeyNotFound,
    #[error("Register response error: {0}")]
    RegisterResponseError(String),
    #[error("SSE error: {0}")]
    SSEError(String),
    #[error("Verify response error: {0}")]
    VerifyResponseError(String),
    #[error("Invalid key")]
    InvalidKey,
}
