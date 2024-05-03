use reqwest::Error as ReqwestError;
use thiserror::Error;
mod models;
mod prove_sdk_builder;
mod prover_sdk;
use hex::FromHexError;
use url::ParseError;

#[derive(Debug, Error)]
enum ProverSdkErrors {
    #[error("HTTP request failed")]
    RequestFailed(#[from] ReqwestError),

    #[error("JSON parsing failed")]
    JsonParsingFailed,

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

    #[error("Signing key not found")]
    SigningKeyNotFound,

    #[error("Nonce request failed")]
    NonceRequestFailed(String),

    #[error("Failed to parse to url")]
    UrlParseError(#[from] ParseError),

    #[error("Failed to decode hex")]
    FromHexError(#[from] FromHexError),
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
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
        assert!(result.is_err(), "Expected authentication to fail with invalid private key");
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
        assert!(result.is_ok(), "Expected authentication to succeed with valid private key");
    }

    #[tokio::test]
    async fn test_proversdk_creation() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";
        let url_auth = "http://localhost:7003/auth";
        let url_prover = "http://localhost:7003/prove/state-diff-commitment";
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex)
            .await?
            .build()?;
        
        // Assert: Check that ProverSDK instance is created successfully
        assert!(true, "ProverSDK instance created successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_prove_method_with_valid_data() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";
        let url_auth = "http://localhost:7003/auth";
        let url_prover = "http://localhost:7003/prove/state-diff-commitment";
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(private_key_hex)
            .await?
            .build()?;
        let data = read_json_file("resources/input.json").await?;

        // Act: Call the prove method with valid input data
        let result = sdk.prove(data).await;

        // Assert: Check that the prove method succeeds
        assert!(result.is_ok(), "Expected the prove method to succeed with valid input data");
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
