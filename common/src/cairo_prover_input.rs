use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

use crate::ProverInput;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CairoProverInput {
    pub program: CairoCompiledProgram,
    pub program_input: Vec<Felt>,
    pub layout: String,
}
impl ProverInput for CairoProverInput {
    fn serialize(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CairoCompiledProgram {
    //pub version: u64,
    pub type_declarations: serde_json::Value,
    pub libfunc_declarations: serde_json::Value,
    pub statements: serde_json::Value,
    pub funcs: serde_json::Value,
}
