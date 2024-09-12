use crate::errors::ProveErrors;
use crate::validate_input;
use crate::Args;
use crate::CairoVersion;
use common::prover_input::*;
use prover_sdk::sdk::ProverSDK;
use serde_json::Value;

pub async fn prove(args: Args, sdk: ProverSDK) -> Result<u64, ProveErrors> {
    let program = std::fs::read_to_string(&args.program_path)?;
    let proof = match args.cairo_version {
        CairoVersion::V0 => {
            let input_path = args
                .program_input_path
                .ok_or(ProveErrors::MissingProgramInput)?;
            let input = std::fs::read_to_string(&input_path)?;
            let program_serialized: Cairo0CompiledProgram = serde_json::from_str(&program)?;
            let program_input: Value = serde_json::from_str(&input)?;
            let data = Cairo0ProverInput {
                program: program_serialized,
                layout: args.layout,
                program_input,
            };
            sdk.prove_cairo0(data).await?
        }
        CairoVersion::V1 => {
            let input = match args.clone().program_input_path {
                Some(input_path) => {
                    let input = std::fs::read_to_string(input_path)?;
                    validate_input(&input)?
                }
                None => args.program_input,
            };
            let program_serialized: CairoCompiledProgram = serde_json::from_str(&program)?;
            let data = CairoProverInput {
                program: program_serialized,
                layout: args.layout,
                program_input: input,
            };
            sdk.prove_cairo(data).await?
        }
    };
    Ok(proof)
}
