use std::{env, path::PathBuf};

use common::{Cairo0ProverInput, Cairo1ProverInput};
use tokio::{fs::File, io::AsyncReadExt};

use crate::errors::ProverSdkErrors;

pub async fn load_cairo0(file_path: PathBuf) -> Result<Cairo0ProverInput, ProverSdkErrors> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut file = File::open(manifest_dir.join(file_path)).await?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).await?;

    let json_value: Cairo0ProverInput = serde_json::from_str(&json_string)?;

    Ok(json_value)
}

pub async fn load_cairo1(file_path: PathBuf) -> Result<Cairo1ProverInput, ProverSdkErrors> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut file = File::open(manifest_dir.join(file_path)).await?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).await?;

    let json_value: Cairo1ProverInput = serde_json::from_str(&json_string)?;

    Ok(json_value)
}
