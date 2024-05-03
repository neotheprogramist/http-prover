use ed25519_dalek::{PublicKey, Signature};
/// Verifies a signature given a nonce and a public key using ed25519_dalek.
///
/// - `signature`: The signature object.
/// - `nonce`: The message that was signed, as a string.
/// - `public_key_hex`: The hexadecimal string of the public key.
///
/// Returns `true` if the signature is valid; `false` otherwise.
pub fn verify_signature(signature: &Signature, nonce: &str, public_key_hex: &str) -> bool {
    let public_key_bytes = match hex::decode(public_key_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let public_key = match PublicKey::from_bytes(&public_key_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    public_key
        .verify_strict(nonce.as_bytes(), &signature)
        .is_ok()
}
