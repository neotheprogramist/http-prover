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
let access_key = ProverAccessKey::from_hex_string(
    "0xf91350db1ca372b54376b519be8bf73a7bbbbefc4ffe169797bc3f5ea2dec740",
)
.unwrap()
```

Then construct an instance with

```rust
let prover_url = Url::parse("http://localhost:3000").unwrap();
let sdk = ProverSDK::new(prover_url, access_key).await?;

```

Then you can use below to prove an execution

```rust
#[derive(Deserialize)]
pub struct JobId {
    pub job_id: u64,
}

data = CairoProverInput{
    program, //CairoCompiledProgram,
    program_input,  //Vec<Felt>,
    layout, //String,
}
let job_id = sdk.prove_cairo(data).await;
let job: JobId = serde_json::from_str(&job_id)?;
sdk.sse(job.job_id).await?;
let response = sdk.get_job(job.job_id).await?;
if let Some(status) = json_response.get("status").and_then(Value::as_str) {
    if status == "Completed" {
        return Ok(json_response
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("No result found")
            .to_string());
    } 
}

```
## Examples

To use the SDK, follow these steps:

Authenticate with the Prover service using your private key and the authentication URL:

```rust
#[tokio::main]
async fn main() -> Result<(), ProverSdkErrors> {
    let sdk = ProverSDK::new(prover_url, access_key).await?;
    Ok(())
}
```

Use the SDK to prove data:

```rust
#[tokio::main]
async fn main() -> Result<(), SdkErrors> {
    // Authentication code goes here...

    let sdk = ProverSDK::new(prover_url, access_key).await?;
    data = CairoProverInput{
    program, //CairoCompiledProgram,
    program_input,  //Vec<Felt>,
    layout, //String,
}
    let job_id = sdk.prove(data).await?; //return job id in json format
    // Handle job id
    Ok(())
}
```
