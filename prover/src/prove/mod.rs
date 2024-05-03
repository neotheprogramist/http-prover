use crate::server::AppState;
use axum::{http::StatusCode, response::IntoResponse, routing::get, routing::post, Json, Router};
use podman::process::ProcessError;
use thiserror::Error;
pub mod models;
mod state_diff_commitment;
use serde_json::json;

#[derive(Error, Debug)]
pub enum ProveError {
    #[error("failed to prove state-diff-commitment")]
    StateDiffCommitment(#[from] ProcessError),

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
}

impl IntoResponse for ProveError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match &self {
            ProveError::StateDiffCommitment(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ProveError::Parse(_) => (StatusCode::BAD_REQUEST, self.to_string()),
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

pub fn router() -> Router {
    Router::new().route("/state-diff-commitment", post(state_diff_commitment::root))
}

pub fn auth(app_state: &AppState) -> Router {
    Router::new()
        .route("/auth", get(crate::auth::generate_nonce))
        .route("/auth", post(crate::auth::validate_signature))
        .with_state(app_state.clone())
}
