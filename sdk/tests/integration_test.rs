mod common;
mod tests {
    use sdk::{load_cairo0, load_cairo1, ProverAccessKey, ProverSDK, ProverSdkErrors};
    use std::path::PathBuf;
    use url::Url;
    use crate::common::spawn_prover;

    // Tests the proving process using the Cairo0 protocol setup.
    #[tokio::test]
    async fn test_prover_cairo0() -> Result<(), ProverSdkErrors> {
        let (handle, key, url) = spawn_prover().await;
        let sdk = ProverSDK::new(key, url).await?;
        let data = load_cairo0(PathBuf::from("../examples/CairoZero/prover_input.json")).await?;
        let proof = sdk.prove_cairo0(data).await;
        assert!(proof.is_ok(), "Failed to prove with invalid url");
        if let Err(err) = proof {
            println!(" error: {}", err);
        }
        handle.abort();
        Ok(())
    }

    // Tests the proving process using the Cairo1 protocol setup.
    #[tokio::test]
    async fn test_prover_cairo1() -> Result<(), ProverSdkErrors> {
        let (handle, key, url) = spawn_prover().await;
        let sdk = ProverSDK::new(key, url).await?;
        let data = load_cairo1(PathBuf::from("../examples/Cairo/prover_input.json")).await?;
        let proof = sdk.prove_cairo1(data).await;
        assert!(proof.is_ok(), "Failed to prove with invalid url");
        if let Err(err) = proof {
            println!(" error: {}", err);
        }
        handle.abort();
        Ok(())
    }

    // Tests authentication failure when connecting to the SDK with an invalid URL.
    #[tokio::test]
    async fn test_invalid_url_auth() -> Result<(), ProverSdkErrors> {
        let (handle, key, _) = spawn_prover().await;
        let prover_url = Url::parse("http://localhost:3345").unwrap();
        let result = ProverSDK::new(key, prover_url).await;
        assert!(
            result.is_err(),
            "Expected authentication to fail with invalid URL for authentication"
        );
        if let Err(err) = result {
            println!("Error message: {}", err);
        }
        handle.abort();
        Ok(())
    }

    // Tests error handling by attempting to use the wrong protocol for proving (Cairo0 instead of Cairo1).
    #[tokio::test]
    async fn test_invalid_prover() -> Result<(), ProverSdkErrors> {
        let (handle, key, url) = spawn_prover().await;
        let sdk = ProverSDK::new(key, url).await?;
        let data = load_cairo1(PathBuf::from("../examples/Cairo/prover_input.json")).await?;
        let proof = sdk.prove_cairo0(data).await;
        assert!(proof.is_err(), "Failed to prove with invalid url");
        handle.abort();
        Ok(())
    }

    // Tests the registration of a new cryptographic key with the prover system.
    #[tokio::test]
    async fn test_register() -> Result<(), ProverSdkErrors> {
        let (handle, key, url) = spawn_prover().await;
        let new_key = ProverAccessKey::generate();
        let mut sdk = ProverSDK::new(key, url).await?;
        sdk.register(new_key.0.verifying_key()).await?;
        handle.abort();
        Ok(())
    }
}
