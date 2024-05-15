use prover::prove::{
    cairo_0_prover_input::Cairo0ProverInput, cairo_1_prover_input::Cairo1ProverInput,
};
use tokio::{fs::File, io::AsyncReadExt};

use crate::errors::ProverSdkErrors;

pub async fn load_cairo0(file_path: &str) -> Result<Cairo0ProverInput, ProverSdkErrors> {
    println!("{:?}", file_path);

    let mut file = File::open(file_path).await?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).await?;

    let json_value: Cairo0ProverInput = serde_json::from_str(&json_string)?;

    Ok(json_value)
}

pub async fn load_cairo1(file_path: &str) -> Result<Cairo1ProverInput, ProverSdkErrors> {
    println!("{:?}", file_path);

    let mut file = File::open(file_path).await?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).await?;

    let json_value: Cairo1ProverInput = serde_json::from_str(&json_string)?;

    Ok(json_value)
}
