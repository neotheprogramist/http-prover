pub mod cairo0_prover_input;
pub mod cairo_prover_input;
pub mod models;
pub mod requests;
pub trait ProverInput {
    fn serialize(self) -> serde_json::Value;
}
