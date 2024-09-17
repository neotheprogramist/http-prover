use clap::{Parser, ValueEnum};
use errors::ProveErrors;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use std::{path::PathBuf, str::FromStr};
use url::Url;

pub mod errors;
pub mod fetch;
pub mod prove;

#[derive(Debug, Serialize, Deserialize, ValueEnum, Clone)]
pub enum CairoVersion {
    V0,
    V1,
}

impl FromStr for CairoVersion {
    type Err = String;

    fn from_str(input: &str) -> Result<CairoVersion, Self::Err> {
        match input {
            "v0" => Ok(CairoVersion::V0),
            "v1" => Ok(CairoVersion::V1),
            _ => Err(format!("Invalid Cairo version: {}", input)),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env)]
    pub prover_url: Url,
    #[arg(long, short, env, default_value = "v1")]
    pub cairo_version: CairoVersion,
    #[arg(long, short, env)]
    pub layout: String,
    #[arg(long, env)]
    pub program_path: PathBuf,
    #[arg(
        long,
        env,
        conflicts_with("program_input"),
        required_if_eq("cairo_version", "v0")
    )]
    pub program_input_path: Option<PathBuf>,
    #[arg(long, env, value_delimiter = ',')]
    pub program_input: Vec<Felt>,
    #[arg(long, env)]
    pub program_output: PathBuf,
    #[arg(long, env)]
    pub prover_access_key: String,
    #[arg(long, env, default_value = "false")]
    pub wait: bool,
    #[arg(long, env, default_value = "false")]
    pub sse: bool,
    #[arg(long, env)]
    pub n_queries: Option<u32>,
    #[arg(long, env)]
    pub pow_bits: Option<u32>,
}

fn validate_input(input: &str) -> Result<Vec<Felt>, ProveErrors> {
    let parts: Vec<&str> = input.split(',').collect();

    let mut felts = Vec::new();
    for part in parts {
        let part = part.replace(['[', '\n', ']'], "");
        match part.trim().parse::<Felt>() {
            Ok(num) => felts.push(num),
            Err(_) => {
                return Err(ProveErrors::Custom(
                    "Input contains non-numeric characters or spaces".to_string(),
                ))
            }
        }
    }
    Ok(felts)
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_validate_input() -> Result<(), ProveErrors> {
        let input = "1,2,3,4,5";
        let result = validate_input(input)?;
        assert_eq!(
            result,
            vec![
                Felt::from(1),
                Felt::from(2),
                Felt::from(3),
                Felt::from(4),
                Felt::from(5)
            ]
        );
        Ok(())
    }
    #[test]
    fn test_validate_input_with_hex() -> Result<(), ProveErrors> {
        let input = "[0x1,0x2,0x3,0x4,0x5]";
        let result = validate_input(input)?;
        assert_eq!(
            result,
            vec![
                Felt::from(1),
                Felt::from(2),
                Felt::from(3),
                Felt::from(4),
                Felt::from(5)
            ]
        );
        Ok(())
    }
    #[test]
    fn test_validate_input_non_numeric() -> Result<(), ProveErrors> {
        let input = "[1,2,a,4,5]";
        let result = validate_input(input);
        println!("{:?}", result);
        assert!(result.is_err());
        Ok(())
    }
}
