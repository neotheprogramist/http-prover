mod cairo;
mod cairo0;
mod pie;
pub use cairo::{CairoCompiledProgram, CairoProverInput};
pub use cairo0::{Cairo0CompiledProgram, Cairo0ProverInput};
pub use pie::PieProverInput;

#[derive(Debug)]
pub enum ProverInput {
    Cairo0(Cairo0ProverInput),
    Cairo(CairoProverInput),
    Pie(PieProverInput),
}

impl ProverInput {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            ProverInput::Cairo0(input) => serde_json::to_value(input).unwrap(),
            ProverInput::Cairo(input) => serde_json::to_value(input).unwrap(),
            ProverInput::Pie(input) => serde_json::to_value(input).unwrap(),
        }
    }
}
