mod cairo;
mod cairo0;

pub use cairo::{CairoCompiledProgram, CairoProverInput};
pub use cairo0::{Cairo0CompiledProgram, Cairo0ProverInput};

#[derive(Debug)]
pub enum ProverInput {
    Cairo0(Cairo0ProverInput),
    Cairo(CairoProverInput),
}

impl ProverInput {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            ProverInput::Cairo0(input) => serde_json::to_value(input).unwrap(),
            ProverInput::Cairo(input) => serde_json::to_value(input).unwrap(),
        }
    }
}
