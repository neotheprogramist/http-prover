use reqwest::Error as ReqwestError;
use thiserror::Error;
mod models;
mod prove_sdk_builder;
mod prover_sdk;
use hex::FromHexError;
use url::ParseError;

#[derive(Debug, Error)]
enum ProverSdkErrors {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] ReqwestError),

    #[error("JSON parsing failed: {0}")]
    JsonParsingFailed(String),

    #[error("Validate signature failed: {0}")]
    ValidateSignatureResponseError(String),

    #[error("Prove request failed: {0}")]
    ProveRequestFailed(String),

    #[error("Prove request failed: {0}")]
    ProveResponseError(String),

    #[error("JSON parsing failed: {0}")]
    ReqwestBuildError(String),
    
    #[error("Failed to serialize")]
    SerdeError(#[from] serde_json::Error),

    #[error("Nonce not found in the response")]
    NonceNotFound,

    #[error("JWT token not found in the response")]
    JwtTokenNotFound,

    #[error("Reading input file failed")]
    ReadFileError(#[from] std::io::Error),

    #[error("Expiration date not found")]
    ExpirationNotFound,

    #[error("Signing key not found, possibly build without auth.")]
    SigningKeyNotFound,

    #[error("Nonce request failed: {0}")]
    NonceRequestFailed(String),

    #[error("Validate signature request failed: {0}")]
    ValidateSignatureRequestFailed(String),

    #[error("Failed to parse to url")]
    UrlParseError(#[from] ParseError),

    #[error("Failed to decode hex")]
    FromHexError(#[from] FromHexError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prover_sdk::ProverSDK;
    use serde_json::Value;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_invalid_private_key_auth() {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "invalid_private_key"; // Provide an invalid private key
        let url_auth = "http://localhost:7003/auth";
        let url_prover = "http://localhost:7003/prove/state-diff-commitment";

        // Act: Attempt to authenticate with the invalid private key
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex)
            .await;

        // Assert: Check that authentication fails
        assert!(
            result.is_err(),
            "Expected authentication to fail with invalid private key"
        );
    }

    #[tokio::test]
    async fn test_prover_without_auth() {
        // Arrange: Set up any necessary data or dependencies
        let url_auth = "http://localhost:7003/auth";
        let url_prover = "http://localhost:7003/prove/state-diff-commitment";

        // Act: Attempt to authenticate with the invalid private key
        let result = ProverSDK::new(url_auth, url_prover).build();

        // Assert: Check that authentication fails
        assert!(
            result.is_err(),
            "Expected authentication to fail because authentication has not been performed"
        );

        // If authentication fails, print out the error message for debugging purposes
        if let Err(err) = result {
            println!("Authentication error: {}", err);
        }
    }


    #[tokio::test]
    async fn test_valid_private_key_auth() {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";
        let url_auth = "http://localhost:7003/auth";
        let url_prover = "http://localhost:7003/prove/state-diff-commitment";

        // Act: Attempt to authenticate with the valid private key
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex)
            .await;

        // Assert: Check that authentication succeeds
        assert!(
            result.is_ok(),
            "Expected authentication to succeed with valid private key"
        );
    }

    #[tokio::test]
    async fn test_invalid_url_auth() {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";
        let url_auth = "invalid_url_auth"; // Provide an invalid URL for authentication
        let url_prover = "http://localhost:7003/prove/state-diff-commitment";
    
        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex)
            .await;
        // Assert: Check that authentication fails due to invalid URL
        assert!(
            result.is_err(),
            "Expected authentication to fail with invalid URL for authentication"
        );
        // If authentication fails, print out the error message
        if let Err(err) = result {
            println!("Error message: {}", err);
        }
    }

    #[tokio::test]
    async fn test_invalid_url_prover()-> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";
        let url_auth =  "http://localhost:7003/auth"; // Provide an invalid URL for authentication
        let url_prover = "http://localhost:7003/prover_invalid";
    
        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex).await?.build()?;
        
        let data = read_json_file("resources/input.json").await?;

        let proof = sdk.prove(data).await;
        // If authentication fails, print out the error message
        assert!(
            proof.is_err(),
            "Failed to prove with invalid url"
        );

        Ok(()) 
    }
    #[tokio::test]
    async fn test_invalid_url_without_base_prover()-> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";
        let url_auth =  "http://localhost:7003/auth"; // Provide an invalid URL for authentication
        let url_prover = "invalid_url_prover";
    
        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex).await?.build();

        assert!(
            sdk.is_err(),
            "Failed to parse url without base to url"
        );

        Ok(()) 
    }
    async fn read_json_file(file_path: &str) -> Result<Value, ProverSdkErrors> {
        println!("{:?}", file_path);

        let mut file = File::open(file_path).await?;
        println!("{:?}", file);
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).await?;

        let json_value: Value = serde_json::from_str(&json_string)?;

        Ok(json_value)
    }
}
