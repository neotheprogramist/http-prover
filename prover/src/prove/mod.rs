use crate::server::AppState;
use axum::{routing::get, routing::post,  Router};
pub mod models;
mod state_diff_commitment;
pub mod errors;

pub fn router() -> Router {
    Router::new().route("/state-diff-commitment", post(state_diff_commitment::root))
}

pub fn auth(app_state: &AppState) -> Router {
    Router::new()
        .route("/auth", get(crate::auth::validation::generate_nonce))
        .route("/auth", post(crate::auth::validation::validate_signature))
        .with_state(app_state.clone())
}


