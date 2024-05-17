mod inputs;
mod models;
mod requests;

pub use inputs::*;
pub use models::*;
pub use requests::*;

pub trait ProverInput {
    fn serialize(self) -> serde_json::Value;
}
