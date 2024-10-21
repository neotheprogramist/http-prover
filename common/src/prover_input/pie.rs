// Module: prover_input/pie.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PieProverInput {
    pub pie_zip: Vec<u8>,
    pub layout: String,
    pub n_queries: Option<u32>,
    pub pow_bits: Option<u32>,
}
