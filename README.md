# Prover SDK

The Prover SDK is a Rust library for interacting with the Prover service. It provides functionality for authentication, proving, and error handling.

## Generating access keys

Before using the prover key has to be authorized by the prover operator. To generate the key use:

```bash
    cargo run --bin keygen
```

It will output 2 keys.

- send the public key to the prover operator
- pass the private key to the sdk to use it.

## Using in code

First parse a private key corresponding to an authorized public key.

```rust
    ProverAccessKey::from_hex_string(
        "0xf91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740",
    )
    .unwrap()

```

Then construct an instance with

```rust
    let prover_url = Url::parse("http://localhost:3000").unwrap();
    let sdk = ProverSDK::new(key, prover_url).await?;

```

Then you can use below to prove an execution

```rust
    let data = load_cairo1(PathBuf::from("../prover/resources/input_cairo1.json")).await?;
    let proof = sdk.prove_cairo1(data).await;
```

# Operating a prover

To run the sdk first make sure prover/authorized_keys.json contains your public_key.

Run the following command in your terminal:

```bash
 cargo run -p prover -- --jwt-secret-key <ENV_VAR_JWT_SECRET_KEY> --message-expiration-time <MESSAGE_EXPIRATION_TIME> --session-expiration-time <SESSION_EXPIRATION_TIME> --authorized-keys <AUTHORIZED_KEY>,<ANOTHER_KEY>
```

Alternatively use the flag `--authorized-keys-path authorized_keys.json` instead of `--authorized-keys` to load keys from a json file. It needs to have the format below

```json
["<AUTHORIZED_KEY>", "<ANOTHER_KEY>"]
```

Note:
Tests from the sdk lib.rs file should be run separately.

# Prover SSL Certificate Automation

The Prover system automatically applies for an SSL certificate and renews it before expiration. The SSL procedure can be configured using environment variables.


Example Environment Variables for Running SSL Procedure
```
# Comma-separated list of domain identifiers in one string
DOMAIN_IDENTIFIERS="example.com,*.example.com"

# Contact email for certificate notifications
CONTACT_MAILS="example@gmail.com"

# Cloudflare API token for managing DNS records
API_TOKEN="-zMM48uvEBakd2gcfOFl4uCJS8YB7OHJE-XzAWXh"

# Cloudflare zone ID for the target domain
ZONE_ID="22bae53e586432f0ea09d615a8f7d510"

# URL for the ACME directory (e.g., Let's Encrypt staging environment)
URL="https://acme-staging-v02.api.letsencrypt.org/directory"

# Logging configuration for tracing the SSL procedure
RUST_LOG="lib_acme=trace,acme_controller=trace,prover=trace"

# File path where the certificate will be saved
CERT_PATH="cert.pem"

# Number of days before the certificate's expiration date when renewal should be triggered
RENEWAL_THRESHOLD="30"
```

## Using sdk

Run command below to generate keys. Pass the public key to operator, after he includes it to the prover you will be able to use sdk.

```bash
cargo run --bin keygen
```

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
