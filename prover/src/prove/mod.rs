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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::Claims;
    use axum::Json;
    use cairo1::root;
    use common::Cairo1ProverInput;
    use errors::ProveError;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_root_with_input_json() {
        // Read input data from resources/input.json
        let input_json: Cairo1ProverInput = read_json_file("../examples/Cairo/fibonacci_prover_input.json")
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

    async fn read_json_file(file_path: &str) -> Result<Cairo1ProverInput, ProveError> {
        let mut file = File::open(file_path).await?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).await?;

        let json_value: Cairo1ProverInput = serde_json::from_str(&json_string)?;

        Ok(json_value)
    }
}
