use axum::{http::StatusCode, response::IntoResponse, Json};
use hex::FromHexError;
use podman::process::ProcessError;
use serde_json::json;
use std::{env::VarError, net::AddrParseError};
use thiserror::Error;

use crate::auth::authorizer::AuthorizerError;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("server error")]
    Server(#[from] std::io::Error),

    #[error("failed to parse address")]
    AddressParse(#[from] AddrParseError),

    #[error("failed to initializer authorizer")]
    AuthorizerCreation(#[from] AuthorizerError),
}

#[derive(Error, Debug)]
pub enum ProveError {
    #[error("failed to prove state-diff-commitment")]
    StateDiffCommitment(#[from] ProcessError),

    #[error("Unauthorized public key request")]
    UnauthorizedPublicKey,

    #[error("HexDecode Error")]
    HexDecodeError(#[from] FromHexError),
    #[error("Failed to read env variable")]
    EnvVarFailed(#[from] VarError),

    #[error("failed to parse result")]
    Parse(#[from] serde_json::Error),

    #[error("unauthorized access")]
    Unauthorized(String),

    #[error("resource not found")]
    NotFound(String),

    #[error("internal server error")]
    InternalServerError(String),

    #[error("Missing or invalid public key")]
    MissingPublicKey,

    #[error(transparent)]
    Auth(#[from] AuthError), // Embedding AuthError within ProveError

    #[error("Failed to parse json")]
    JsonParsingFailed(String),

    #[error("File read error")]
    FileReadError(#[from] std::io::Error),

    #[error("Authorization failure")]
    AuthorizationFailure(#[from] AuthorizerError),
}

impl IntoResponse for ProveError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match &self {
            ProveError::StateDiffCommitment(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ProveError::Parse(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ProveError::UnauthorizedPublicKey => (StatusCode::UNAUTHORIZED, self.to_string()),
            ProveError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            ProveError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ProveError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ProveError::MissingPublicKey => (StatusCode::BAD_REQUEST, self.to_string()),
            ProveError::Auth(auth_error) => match auth_error {
                AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
                AuthError::MissingAuthorizationHeader => (
                    StatusCode::UNAUTHORIZED,
                    "Missing authorization header".to_string(),
                ),
                AuthError::Unauthorized => {
                    (StatusCode::UNAUTHORIZED, "Unauthorized access".to_string())
                }
            },
            ProveError::FileReadError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ProveError::JsonParsingFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ProveError::EnvVarFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ProveError::HexDecodeError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ProveError::AuthorizationFailure(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
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
