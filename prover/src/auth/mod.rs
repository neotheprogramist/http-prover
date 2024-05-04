pub mod jwt;
pub mod validation;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use axum::extract::Extension;
    use axum::body::Body;
    use axum::http::{Response, StatusCode};
    use crate::prove::errors::ProveError;
    use crate::server::AppState;
    use crate::prove::models::GenerateNonceRequest;
    use crate::auth::validation::generate_nonce;
    use crate::auth::validation::is_public_key_authorized;
    use crate::prove::models::ValidateSignatureRequest;
    use crate::auth::validation::validate_signature;
    use ed25519_dalek::Signature;
    use axum::extract::State;
    use axum::extract::Query;
    use std::sync::Arc;
    use std::sync::Mutex;
    use axum::Json;
    #[tokio::test]
    async fn test_generate_nonce() ->Result<(),ProveError>{
        let state = AppState {
            nonces: Arc::new(Mutex::new(HashMap::new())),
            prover_image_name:"sample".to_string()
        };
        let params = GenerateNonceRequest {
            public_key: "05a257b53c49a28f2eb391653695e3ad2964ccec11fb30ca2b3d334187985501".to_string(),
        };
        let result = generate_nonce(State(state.into()), Query(params)).await;
 
        assert!(result.is_ok());

        let response = result.unwrap();

        println!("{:?}",response);
        Ok(())
    }


    #[tokio::test]
    async fn test_is_public_key_authorized() {
        // Test with an authorized key
        let result = is_public_key_authorized("authorized_keys.json", "05a257b53c49a28f2eb391653695e3ad2964ccec11fb30ca2b3d334187985501").await;
        assert!(result.is_ok());

        // Test with an unauthorized key
        let result = is_public_key_authorized("authorized_keys.json", "unauthorized_key").await;
        assert!(result.is_err());
    }
}
