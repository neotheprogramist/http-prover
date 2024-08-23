pub use access_key::ProverAccessKey;
pub use common::{Cairo0ProverInput, Cairo1CompiledProgram, Cairo1ProverInput, CompiledProgram};
pub use errors::ProverSdkErrors;
pub use load::{load_cairo0, load_cairo1};
pub use prover_sdk::ProverSDK;
use prover_sdk::{access_key, errors, load};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::load::{load_cairo0, load_cairo1};
    use crate::ProverAccessKey;
    use prover_sdk::{ProverSDK, ProverSdkErrors};
    use url::Url;

    fn get_signing_key() -> ProverAccessKey {
        ProverAccessKey::from_hex_string(
            "0x5883b0e30b008e48af3d0bf5cfc138fb6093496da6f87d24b65def88470356d3",
            // Corresponding to 0xd16b71c90dbf897e5964d2f267d04664b3c035036559d712994739ea6cf2fd9f public key.
        )
        .unwrap()
    }
    #[tokio::test]
    async fn test_prover_cairo0() -> Result<(), ProverSdkErrors> {
        let prover_url = Url::parse("http://localhost:3040").unwrap(); // Provide an invalid URL
        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        let data = load_cairo0(PathBuf::from("../examples/CairoZero/prover_input.json")).await?;
        let proof = sdk.prove_cairo0(data).await;
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
        let prover_url = Url::parse("http://localhost:3040").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        let data = load_cairo1(PathBuf::from(
            "../examples/Cairo/fibonacci_prover_input.json",
        ))
        .await?;
        let proof = sdk.prove_cairo1(data).await;
        // If authentication fails, print out the error message
        assert!(proof.is_ok(), "Failed to prove with invalid url");
        // If authentication fails, print out the error message for debugging purposes
        if let Err(err) = proof {
            println!(" error: {}", err);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_url_auth() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let prover_url = Url::parse("http://wrong:1234").unwrap(); // Provide an invalid URL for authentication

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let result = ProverSDK::new(get_signing_key(), prover_url).await;
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
    async fn test_invalid_prover() -> Result<(), ProverSdkErrors> {
        // Arrange: Set up any necessary data or dependencies
        let prover_url: Url = Url::parse("http://localhost:3040").unwrap();

        // Act: Attempt to authenticate with the valid private key and valid URL for authentication
        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;
        //Act: Load wrong prover input to test invalid prover version

        let data = load_cairo1(PathBuf::from(
            "../examples/Cairo/fibonacci_prover_input.json",
        ))
        .await?;

        let proof = sdk.prove_cairo0(data).await;
        // If authentication fails, print out the error message
        assert!(proof.is_err(), "Failed to prove with invalid url");
        Ok(())
    }

    #[tokio::test]
    async fn test_register() -> Result<(), ProverSdkErrors> {
        let prover_url: Url = Url::parse("http://localhost:3040").unwrap();
        let new_key = ProverAccessKey::generate();

        // Act: Attempt to authenticate with the valid private key
        let mut sdk = ProverSDK::new(get_signing_key(), prover_url).await?;
        sdk.register(new_key.0.verifying_key()).await?;

        // If authentication fails, print out the error message

        Ok(())
    }
}
