mod inputs;
mod models;

pub use inputs::*;
pub use models::*;

pub trait ProverInput {
    fn serialize(self) -> serde_json::Value;
}
