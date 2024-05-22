use std::path::PathBuf;

use prove::{prove, CliInput};

use crate::common::{read_file, spawn_prover};

mod common;

#[tokio::test]
async fn test_cairo1_differ() -> Result<(), prove::ProveError> {
    let (handle, key, url) = spawn_prover().await;

    let args = CliInput {
        prover_key: key.signing_key_as_hex_string(),
        cairo_version: 1,
        prover_url: url,
    };

    let prover_input = read_file(PathBuf::from("input_cairo1_differ.json")).await?;
    prove(args, prover_input).await?;

    handle.abort();
    Ok(())
}
