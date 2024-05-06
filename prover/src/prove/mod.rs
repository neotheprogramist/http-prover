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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use errors::ProveError;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use state_diff_commitment::root;
    use crate::auth::jwt::Claims;
    use std::env;
    #[tokio::test]
    async fn test_root_with_input_json() {
        // Read input data from resources/input.json
        let input_json = read_json_file("resources/input.json").await.expect("Failed to read input JSON");

        // Call the root function with the input data and actual PodmanRunner
        let result = root(Claims {sub:"jwt_token".to_string(),exp:3600}, input_json.to_string()).await;

        // Check if the result is as expected
        assert!(result.is_ok());
        // Add assertions based on the expected behavior of root function
    }

    async fn read_json_file(file_path: &str) -> Result<Value, ProveError> {
        println!("{:?}", file_path);

        let mut file = File::open(file_path).await?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).await?;

        let json_value: Value = serde_json::from_str(&json_string)?;

        Ok(json_value)
    }

}