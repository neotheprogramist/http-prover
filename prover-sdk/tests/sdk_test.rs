pub use access_key::ProverAccessKey;
pub use common::{Cairo0ProverInput, Cairo1CompiledProgram, Cairo1ProverInput, CompiledProgram};
pub use errors::ProverSdkErrors;
pub use load::{load_cairo0, load_cairo1};
pub use prover_sdk::ProverSDK;
use prover_sdk::{access_key, errors, load};

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use crate::load::{load_cairo0, load_cairo1};
    use crate::ProverAccessKey;
    use prover_sdk::{ProverSDK, ProverSdkErrors};
    use url::Url;

    fn get_signing_key() -> ProverAccessKey {
        ProverAccessKey::from_hex_string(
            "0x5883b0e30b008e48af3d0bf5cfc138fb6093496da6f87d24b65def88470356d3",
        )
        .unwrap()
    }
    #[tokio::test]
    async fn test_prover_cairo0() -> Result<(), ProverSdkErrors> {
        let port = env::var("PORT").unwrap();
        let prover_url = Url::parse(&format!("http://localhost:{}", port)).unwrap();

        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        let data = load_cairo0(PathBuf::from("../examples/CairoZero/prover_input.json")).await?;
        let proof = sdk.prove_cairo0(data).await;

        assert!(proof.is_ok(), "Failed to generate proof with Cairo 0");

        // Verify the generated proof if successful
        if let Err(err) = proof {
            println!("Error during proof generation: {}", err);
        } else {
            let result: Result<String, ProverSdkErrors> = sdk.verify(proof.unwrap()).await;
            assert!(result.is_ok(), "Failed to verify proof");
            assert_eq!(result.unwrap(), "true", "Proof verification failed");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_invalid_proof() {
        let port = env::var("PORT").unwrap();
        let prover_url = Url::parse(&format!("http://localhost:{}", port)).unwrap();

        let sdk = ProverSDK::new(get_signing_key(), prover_url).await.unwrap();

        // Attempt to verify an invalid proof
        let result = sdk.verify("invalid_proof".to_string()).await;

        assert!(
            result.is_ok(),
            "Verification unexpectedly failed for an invalid proof"
        );
        assert_eq!(
            result.unwrap(),
            "false",
            "Invalid proof was incorrectly verified as valid"
        );
    }

    #[tokio::test]
    async fn test_prover_cairo1() -> Result<(), ProverSdkErrors> {
        let port = env::var("PORT").unwrap();
        let prover_url = Url::parse(&format!("http://localhost:{}", port)).unwrap();

        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        let data = load_cairo1(PathBuf::from(
            "../examples/Cairo/fibonacci_prover_input.json",
        ))
        .await?;
        let proof = sdk.prove_cairo1(data).await;

        assert!(proof.is_ok(), "Failed to generate proof with Cairo 1");

        if let Err(err) = proof {
            println!("Error during proof generation: {}", err);
        } else {
            let result: Result<String, ProverSdkErrors> = sdk.verify(proof.unwrap()).await;
            assert!(result.is_ok(), "Failed to verify proof");
            assert_eq!(result.unwrap(), "true", "Proof verification failed");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_url_auth() -> Result<(), ProverSdkErrors> {
        // Provide an invalid URL for SDK initialization
        let prover_url = Url::parse("http://wrong:1234").unwrap();

        let result = ProverSDK::new(get_signing_key(), prover_url).await;

        // Assert that SDK initialization fails due to the invalid URL
        assert!(
            result.is_err(),
            "Expected SDK initialization to fail with an invalid URL"
        );

        if let Err(err) = result {
            println!("Error during SDK initialization: {}", err);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_prover_data() -> Result<(), ProverSdkErrors> {
        let port = env::var("PORT").unwrap();
        let prover_url = Url::parse(&format!("http://localhost:{}", port)).unwrap();

        let sdk = ProverSDK::new(get_signing_key(), prover_url).await?;

        // Load Cairo 1 prover input data (intentional mismatch)
        let data = load_cairo1(PathBuf::from(
            "../examples/Cairo/fibonacci_prover_input.json",
        ))
        .await?;

        // Attempt to prove using Cairo 0 with incorrect input
        let proof = sdk.prove_cairo0(data).await;

        // Assert that proof generation fails due to incorrect input
        assert!(
            proof.is_err(),
            "Expected proof generation to fail with incorrect input"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_register() -> Result<(), ProverSdkErrors> {
        let port = env::var("PORT").unwrap();
        let prover_url = Url::parse(&format!("http://localhost:{}", port)).unwrap();

        let new_key = ProverAccessKey::generate();

        // Initialize the SDK and register the new key
        let mut sdk = ProverSDK::new(get_signing_key(), prover_url).await?;
        sdk.register(new_key.0.verifying_key()).await?;

        Ok(())
    }
}
