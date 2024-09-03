pub mod auth_errors;
pub mod authorizer;
pub mod jwt;
pub mod nonce;
pub mod register;
pub mod validation;
use crate::server::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use nonce::generate_nonce;
use register::register;
use validation::validate_signature;

pub fn auth(app_state: AppState) -> Router {
    Router::new()
        .route("/auth", get(generate_nonce))
        .route("/auth", post(validate_signature))
        .route("/register", post(register))
        .with_state(app_state.clone())
}
