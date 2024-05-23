mod access_key;
mod errors;
mod load;
pub mod prove_sdk_builder;
mod prover_sdk;

pub use access_key::ProverAccessKey;
pub use common::{Cairo0ProverInput, Cairo1ProverInput};
pub use errors::ProverSdkErrors;
pub use load::{load_cairo0, load_cairo1};
pub use prover_sdk::ProverSDK;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::errors::ProverSdkErrors;
    use crate::load::{load_cairo0, load_cairo1};
    use crate::prover_sdk::ProverSDK;
    use crate::ProverAccessKey;
    use prover::server::start;
    use prover::Args;
    use tokio::task::JoinHandle;
    use url::Url;

    fn get_signing_key() -> ProverAccessKey {
        ProverAccessKey::from_hex_string(
            "afe68fcd03b68a4b3dcd6d591d77afd81587189a4e68b2f9ac7e50bd50063ade",
            // Corresponding to 76b4680bd6cf336789a06ccf100c1601fca5e8c41b2d87c784f7f5953a4d346e public key.
        )
        .unwrap()
    }

    async fn spawn_prover() -> (JoinHandle<()>, ProverAccessKey) {
        let key = ProverAccessKey::generate();
        let encoded_key = prefix_hex::encode(key.0.verifying_key().to_bytes());

        let args = Args {
            host: "0.0.0.0".to_string(),
            port: 3000,
            jwt_secret_key: "placeholder".to_string(),
            message_expiration_time: 60,
            session_expiration_time: 3600,
            authorized_keys: Some(vec![encoded_key]),
            authorized_keys_path: None,
        };

        let handle = tokio::spawn(async move {
            start(args).await.unwrap();
        });

        (handle, key)
    }

    #[tokio::test]
    async fn test_prover_cairo1_spawn_prover() -> Result<(), ProverSdkErrors> {
        let (_handle, key) = spawn_prover().await;

        let prover_url = Url::parse("http://localhost:3000").unwrap();
        let sdk = ProverSDK::new(key, prover_url).await?;

        let data = load_cairo1(PathBuf::from("../examples/Cairo/prover_input.json")).await?;
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
    async fn test_prover_cairo0() -> Result<(), ProverSdkErrors> {
        let prover_url = Url::parse("http://localhost:3000").unwrap(); // Provide an invalid URL
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
        let prover_url = Url::parse("http://localhost:3000").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        let data = load_cairo1(PathBuf::from("../examples/Cairo/prover_input.json")).await?;
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
        let prover_url = Url::parse("http://localhost:3345").unwrap(); // Provide an invalid URL for authentication

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
        let prover_url: Url = Url::parse("http://localhost:3000").unwrap();

        // Act: Attempt to authenticate with the valid private key and invalid URL for authentication
        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        let data = load_cairo1(PathBuf::from("../examples/Cairo/prover_input.json")).await?;

        let proof = sdk.prove_cairo0(data).await;
        // If authentication fails, print out the error message
        assert!(proof.is_err(), "Failed to prove with invalid url");

        Ok(())
    }

    #[tokio::test]
    async fn test_register() -> Result<(), ProverSdkErrors> {
        let prover_url: Url = Url::parse("http://localhost:3000").unwrap();
        let new_key = ProverAccessKey::generate();

        // Act: Attempt to authenticate with the valid private key
        let mut sdk = ProverSDK::new(get_signing_key(), prover_url).await?;
        sdk.register(new_key.0.verifying_key()).await?;
        // If authentication fails, print out the error message

        Ok(())
    }
}
