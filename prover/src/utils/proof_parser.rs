use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use swiftness_proof_parser::StarkProof;

pub fn extract_program_hash(stark_proof: StarkProof) -> Felt {
    let program_output_range = &stark_proof.public_input.segments[2];
    let main_page_len = stark_proof.public_input.main_page.len();
    let output_len = (program_output_range.stop_ptr - program_output_range.begin_addr) as usize;
    let program = stark_proof.public_input.main_page[0..main_page_len - output_len].to_vec();

    let values: Vec<Felt> = program
        .iter()
        .map(|el| {
            let number = &el.value;

            let mut padded_bytes = [0u8; 32];
            let bytes = number.to_bytes_be();

            let bytes_len = bytes.len();

            padded_bytes[32 - bytes_len..].copy_from_slice(&bytes);

            Felt::from_bytes_be(&padded_bytes)
        })
        .collect();
    poseidon_hash_many(&values)
}
pub fn extract_program_output(stark_proof: StarkProof) -> Vec<Felt> {
    let program_output_range = &stark_proof.public_input.segments[2];
    let main_page_len = stark_proof.public_input.main_page.len();
    let output_len = (program_output_range.stop_ptr - program_output_range.begin_addr) as usize;
    let program_output = stark_proof.public_input.main_page[main_page_len - output_len..].to_vec();
    let values: Vec<Felt> = program_output
        .iter()
        .map(|el| {
            let number = &el.value;

            let mut padded_bytes = [0u8; 32];
            let bytes = number.to_bytes_be();

            let bytes_len = bytes.len();

            padded_bytes[32 - bytes_len..].copy_from_slice(&bytes);

            Felt::from_bytes_be(&padded_bytes)
        })
        .collect();
    values
}
pub fn program_output_hash(felts: Vec<Felt>) -> Felt {
    poseidon_hash_many(&felts)
}
