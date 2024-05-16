use crate::server::AppState;
use axum::{routing::get, routing::post, Router};
mod cairo0;
mod cairo1;
pub mod errors;
pub mod models;

pub fn auth(app_state: &AppState) -> Router {
    Router::new()
        .route("/auth", get(crate::auth::validation::generate_nonce))
        .route("/auth", post(crate::auth::validation::validate_signature))
        .route(
            "/register",
            post(crate::auth::validation::add_authorized_key),
        )
        .with_state(app_state.clone())
}
pub fn router(app_state: &AppState) -> Router {
    Router::new()
        .route("/cairo0", post(cairo0::root))
        .route("/cairo1", post(cairo1::root))
        .with_state(app_state.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::Claims;
    use axum::Json;
    use cairo0::root;
    use common::Cairo0ProverInput;
    use errors::ProveError;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_root_with_input_json() {
        // Read input data from resources/input.json
        let input_json = read_json_file("resources/input_cairo0.json")
            .await
            .expect("Failed to read input JSON");

        // Call the root function with the input data and actual PodmanRunner
        let result = root(
            Claims {
                sub: "jwt_token".to_string(),
                exp: 3600,
            },
            Json(input_json),
        )
        .await;

        // Check if the result is as expected
        assert!(result.is_ok());
        // Add assertions based on the expected behavior of root function
    }

    async fn read_json_file(file_path: &str) -> Result<Cairo0ProverInput, ProveError> {
        let mut file = File::open(file_path).await?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).await?;

        let json_value: Cairo0ProverInput = serde_json::from_str(&json_string)?;

        Ok(json_value)
    }
}
