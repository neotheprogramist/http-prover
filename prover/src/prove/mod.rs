use crate::server::AppState;
use axum::{routing::post, Router};
mod cairo0;
mod cairo1;
pub mod errors;
pub mod models;

pub fn router(app_state: &AppState) -> Router {
    Router::new()
        .route("/cairo0", post(cairo0::root))
        .route("/cairo1", post(cairo1::root))
        .with_state(app_state.clone())
}
