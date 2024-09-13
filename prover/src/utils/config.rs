use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use crate::errors::ProverError;

#[derive(Serialize, Deserialize, Debug)]
struct StarkFri {
    fri_step_list: Vec<u32>,
    last_layer_degree_bound: u32,
    n_queries: u32,
    proof_of_work_bits: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stark {
    fri: StarkFri,
    log_n_cosets: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    field: String,
    channel_hash: String,
    commitment_hash: String,
    n_verifier_friendly_commitment_layers: u32,
    pow_hash: String,
    statement: Value,
    stark: Stark,
    use_extension_field: bool,
    verifier_friendly_channel_updates: bool,
    verifier_friendly_commitment_hash: String,
}

impl Template {
    pub fn generate_from_public_input_file(file: &PathBuf) -> Result<Self, ProverError> {
        Self::generate_from_public_input(ProgramPublicInputAsNSteps::read_from_file(file)?)
    }
    pub fn save_to_file(&self, file: &PathBuf) -> Result<(), ProverError> {
        let json_string = serde_json::to_string_pretty(self)?;
        File::create(file)?
            .write_all(json_string.as_bytes())
            .map_err(ProverError::from)
    }
    fn generate_from_public_input(
        public_input: ProgramPublicInputAsNSteps,
    ) -> Result<Self, ProverError> {
        let mut template = Self::default();
        let fri_step_list =
            public_input.calculate_fri_step_list(template.stark.fri.last_layer_degree_bound);
        template.stark.fri.fri_step_list = fri_step_list;
        Ok(template)
    }
}

impl core::default::Default for Template {
    fn default() -> Self {
        Template {
            field: "PrimeField0".to_string(),
            channel_hash: "poseidon3".to_string(),
            commitment_hash: "blake256_masked160_lsb".to_string(),
            n_verifier_friendly_commitment_layers: 9999,
            pow_hash: "keccak256".to_string(),
            statement: serde_json::json!({ "page_hash": "pedersen" }),
            stark: Stark {
                fri: StarkFri {
                    fri_step_list: vec![0, 4, 4, 4],
                    last_layer_degree_bound: 128,
                    n_queries: 8,
                    proof_of_work_bits: 30,
                },
                log_n_cosets: 3,
            },
            use_extension_field: false,
            verifier_friendly_channel_updates: true,
            verifier_friendly_commitment_hash: "poseidon3".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProgramPublicInputAsNSteps {
    n_steps: u32,
}

impl ProgramPublicInputAsNSteps {
    pub fn read_from_file(input_file: &PathBuf) -> Result<Self, ProverError> {
        serde_json::from_reader(BufReader::new(File::open(input_file)?)).map_err(ProverError::from)
    }
    fn calculate_fri_step_list(&self, degree_bound: u32) -> Vec<u32> {
        let fri_degree = ((self.n_steps as f64 / degree_bound as f64).log(2.0).round() as u32) + 4;
        let mut steps = vec![0];
        steps.extend(vec![4; (fri_degree / 4) as usize]);
        if fri_degree % 4 != 0 {
            steps.push(fri_degree % 4);
        }
        steps
    }
}
