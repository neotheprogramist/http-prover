# Prover SDK

The Prover SDK is a Rust library for interacting with the Prover service. It provides functionality for authentication, proving, and error handling.

## Installation

To run the sdk first make sure prover/authorized_keys.json contains your public_key.

Run the following command in your terminal:

```bash
 cargo run -p prover -- --jwt-secret-key <ENV_VAR_JWT_SECRET_KEY> --message-expiration-time <MESSAGE_EXPIRATION_TIME> --session-expiration-time <SESSION_EXPIRATION_TIME> --private-key <PRIVATE_KEY>
```

Note:
Tests from the sdk lib.rs file should be run separately.


## Usage

To use the SDK, follow these steps:

Authenticate with the Prover service using your private key and the authentication URL:
```
    #[tokio::main]
    async fn main() -> Result<(), ProverSdkErrors> {
        let private_key_hex : String= env::var("PRIVATE_KEY")?;
        let url_auth =  "http://localhost:3000/auth";
        let url_prover = "http://localhost:3000/prove/cairo1";

        let result = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await?;

        // Handle authentication result
        Ok(())
    }
```
Use the SDK to prove data:

```
    #[tokio::main]
    async fn main() -> Result<(), ProverSdkErrors> {
        // Authentication code goes here...

        let sdk = result.build()?;
        let data = read_json_file("resources/input.json").await?;
        let proof = sdk.prove(data).await?;

        // Handle proof result
        Ok(())
    }
```

Handle errors using the provided error types:

```
    #[tokio::main]
    async fn main() -> Result<(), ProverSdkErrors> {
        // Authentication code goes here...

        let result = ProverSDK::new(url_auth, url_prover)
            .auth(&private_key_hex)
            .await;

        match result {
            Ok(sdk) => {
                // Continue with SDK usage...
            }
            Err(err) => {
                // Handle authentication error
                println!("Authentication failed: {}", err);
            }
        }

        Ok(())
    }
```