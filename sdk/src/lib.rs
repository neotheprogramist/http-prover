mod access_key;
mod load;
pub mod prove_sdk_builder;
pub mod prover_sdk;

pub mod errors;

pub use access_key::ProverAccessKey;
pub use load::{load_cairo0, load_cairo1};

#[cfg(test)]
mod tests {
    use crate::errors::ProverSdkErrors;
    use crate::load::{load_cairo0, load_cairo1};
    use crate::prover_sdk::ProverSDK;
    use crate::ProverAccessKey;
    use url::Url;

    fn get_signing_key() -> ProverAccessKey {
        ProverAccessKey::from_hex_string(
            "f91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740",
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_prover_cairo0() -> Result<(), ProverSdkErrors> {
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap(); // Provide an invalid URL for authentication
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(get_signing_key())
            .await?
            .build()?;
        let data = load_cairo0("../prover/resources/input_cairo0.json").await?;
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
    async fn test_prover_cairo1() -> Result<(), ProverSdkErrors> {
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap(); // Provide an invalid URL for authentication
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(get_signing_key())
            .await?
            .build()?;

        let data = load_cairo1("../prover/resources/input_cairo1.json").await?;
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
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap();
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

        // Act: Attempt to authenticate with the invalid private key
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(ProverAccessKey::random())
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
    async fn test_prover_without_auth() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap();
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

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

        Ok(())
    }

    #[tokio::test]
    async fn test_valid_private_key_auth() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap();
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

        // Act: Attempt to authenticate with the valid private key
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(get_signing_key())
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
        let url_auth = Url::parse("http://localhost:3000/notauth").unwrap(); // Provide an invalid URL for authentication
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let result = ProverSDK::new(url_auth, url_prover)
            .auth(get_signing_key())
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
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap();
        let url_prover = Url::parse("http://localhost:3000/prove/cairo0").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(get_signing_key())
            .await?
            .build()?;

        let data = load_cairo0("../prover/resources/input.json").await?;

        let proof = sdk.prove(data).await;
        // If authentication fails, print out the error message
        assert!(proof.is_err(), "Failed to prove with invalid url");

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_url_without_base_prover() -> Result<(), ProverSdkErrors> {
        let url_auth = Url::parse("http://localhost:3000/auth").unwrap();
        let url_prover = Url::parse("http://localhost:3000/notprove/cairo0").unwrap(); // Provide an invalid URL for authentication

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(url_auth, url_prover)
            .auth(get_signing_key())
            .await?
            .build();

        assert!(sdk.is_err(), "Failed to parse url without base to url");

        Ok(())
    }
}
