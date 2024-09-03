use prover_sdk::access_key::ProverAccessKey;

fn main() {
    let key = ProverAccessKey::generate();
    println!(
        "Public key:  {}, provide it to the server operator.",
        key.verifying_key_as_hex_string()
    );

    println!(
        "Private key: {}, pass this to the sdk to gain access, keep it secret.",
        key.signing_key_as_hex_string()
    )
}
