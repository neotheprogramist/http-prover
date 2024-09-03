use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthorizerError {
    #[error("file read error")]
    FileAccessError(#[from] std::io::Error),
    #[error("invalid file format")]
    FormatError(#[from] serde_json::Error),
    #[error("Missing authorization header")]
    MissingAuthorizationHeader,
    #[error("Conversion from prefix hex failed: {0}")]
    PrefixHexConversionError(String),
    #[error(transparent)]
    VerifyingKeyError(#[from] ed25519_dalek::SignatureError),
    #[error("Unexpected data error: {0:?}")]
    DataError(Vec<u8>),
}

impl From<Vec<u8>> for AuthorizerError {
    fn from(err: Vec<u8>) -> Self {
        AuthorizerError::DataError(err)
    }
}
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing authorization header")]
    MissingAuthorizationHeader,

    #[error("Unauthorized")]
    Unauthorized,
}
