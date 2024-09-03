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
data = CairoProverInput{
    program, //CairoCompiledProgram,
    program_input,  //Vec<Felt>,
    layout, //String,
}
let proof = sdk.prove_cairo1(data).await;
```
## Examples

To use the SDK, follow these steps:

Authenticate with the Prover service using your private key and the authentication URL:

```rust
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

```rust
#[tokio::main]
async fn main() -> Result<(), SdkErrors> {
    // Authentication code goes here...

    let sdk = result.build()?;
    let data = read_json_file("resources/input.json").await?;
    let job_id = sdk.prove(data).await?; //return job id in json format
    // Handle job id
    Ok(())
}
```

Handle errors using the provided error types:

```rust
#[tokio::main]
async fn main() -> Result<(), SdkErrors> {
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