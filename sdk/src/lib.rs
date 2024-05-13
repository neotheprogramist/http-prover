mod models;
mod prove_sdk_builder;
mod prover_sdk;

pub mod errors;

#[cfg(test)]
mod tests {
    use crate::errors::ProverSdkErrors;
    use crate::prover_sdk::ProverSDK;
    use prover::prove::prove_input::ProveInput;
    use std::env;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use tracing::debug;
    //Note: Run tests separately because all are async

    #[tokio::test]
    async fn test_prover() -> Result<(), ProverSdkErrors> {
        let private_key_hex: String = env::var("PRIVATE_KEY")?;
        let url_auth = "http://localhost:3000/auth"; // Provide an invalid URL for authentication
        let url_prover = "http://localhost:3000/prove/cairo0-prove";
        
        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await?
            .build()?;
        let data = read_json_file("../prover/resources/input.json").await?;
        let proof = sdk.prove(data).await;
        // If authentication fails, print out the error message
        assert!(proof.is_ok(), "Failed to prove with invalid url");
        // If authentication fails, print out the error message for debugging purposes
        if let Err(err) = proof {
            println!(" error: {}", err);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_private_key_auth() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex: String = "invalid_key".to_string();
        let url_auth = "http://localhost:3000/auth";
        let url_prover = "http://localhost:3000/prove/cairo0-prove";

        // Act: Attempt to authenticate with the invalid private key
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await;

        // Assert: Check that authentication fails
        assert!(
            result.is_err(),
            "Expected authentication to fail with invalid private key"
        );
        if let Err(err) = result {
            println!("Authentication error: {}", err);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_prover_without_auth() {
        // Arrange: Set up any necessary data or dependencies
        let url_auth = "http://localhost:3000/auth";
        let url_prover = "http://localhost:3000/prove/cairo0-prove";

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
    async fn test_valid_private_key_auth() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex: String = env::var("PRIVATE_KEY")?;
        let url_auth = "http://localhost:3000/auth";
        let url_prover = "http://localhost:3000/prove/cairo0-prove";

        // Act: Attempt to authenticate with the valid private key
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await;

        // Assert: Check that authentication succeeds
        assert!(
            result.is_ok(),
            "Expected authentication to succeed with valid private key"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_url_auth() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex: String = env::var("PRIVATE_KEY")?;
        let url_auth = "invalid_url_auth"; // Provide an invalid URL for authentication
        let url_prover = "http://localhost:3000/prove/cairo0-prove";

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
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
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_url_prover() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let private_key_hex: String = env::var("PRIVATE_KEY")?;
        let url_auth = "http://localhost:3000/auth"; // Provide an invalid URL for authentication
        let url_prover = "http://localhost:3000/prover_invalid";

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await?
            .build()?;

        let data = read_json_file("../prover/resources/input.json").await?;

        let proof = sdk.prove(data).await;
        // If authentication fails, print out the error message
        assert!(proof.is_err(), "Failed to prove with invalid url");

        Ok(())
    }
    #[tokio::test]
    async fn test_invalid_url_without_base_prover() -> Result<(), ProverSdkErrors> {
        let private_key_hex: String = env::var("PRIVATE_KEY")?;
        let url_auth = "http://localhost:3000/auth"; // Provide an invalid URL for authentication
        let url_prover = "invalid_url_prover";

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await?
            .build();

        assert!(sdk.is_err(), "Failed to parse url without base to url");

        Ok(())
    }
    async fn read_json_file(file_path: &str) -> Result<ProveInput, ProverSdkErrors> {
        println!("{:?}", file_path);

        let mut file = File::open(file_path).await?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).await?;

        let json_value: ProveInput = serde_json::from_str(&json_string)?;

        Ok(json_value)
    }
}
