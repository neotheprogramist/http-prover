use reqwest::Error as ReqwestError;
use thiserror::Error;
mod models;
mod prover_sdk;
use url:: ParseError;


#[derive(Debug,Error)]
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
    #[error("Failed to parse to url")]
    UrlParseError(#[from] ParseError),
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prover_sdk::ProverSDK;
    use tokio::fs::File;
    use serde_json::Value;
    use tokio::io::AsyncReadExt;
    
    #[tokio::test]
    async fn test_prover_sdk() ->Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex = "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740";

        // Act: Call the method under test
        // Note: This code assumes `auth()` and `prove()` methods return `Result` types.
        let sdk = ProverSDK::new().auth(private_key_hex).await?.build()?;
        let data = read_json_file("resources/input.json").await?;
        let result = sdk.prove(data).await;
        // Assert: Check the result
        assert!(result.is_ok(), "Failed to create ProverSDK: {:?}", result);

        // Additional assertions can be added based on the behavior you want to test
        Ok(())
    }
    async fn read_json_file(file_path: &str) -> Result<Value, ProverSdkErrors> {
        let mut file = File::open(file_path).await?;
    
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).await?;
    
        let json_value: Value = serde_json::from_str(&json_string)?;
    
        Ok(json_value)
    }
}


 

