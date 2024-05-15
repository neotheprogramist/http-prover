use serde::{Deserialize, Serialize};

use crate::ProverInput;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cairo1ProverInput {
    pub program: Cairo1CompiledProgram,
    pub program_input: serde_json::Value,
}
impl ProverInput for Cairo1ProverInput {
    fn serialize(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cairo1CompiledProgram {
    pub version: u64,
    pub type_declarations: serde_json::Value,
    pub libfunc_declarations: serde_json::Value,
    pub statements: serde_json::Value,
    pub funcs: serde_json::Value,
}
