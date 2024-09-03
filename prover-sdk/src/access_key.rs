use ed25519_dalek::SigningKey;

use crate::errors::SdkErrors;

#[derive(Debug, Clone)]
pub struct ProverAccessKey(pub SigningKey);

impl ProverAccessKey {
    pub fn new(private_key: SigningKey) -> Self {
        Self(private_key)
    }

    pub fn from_hex_string(hex_string: &str) -> Result<Self, SdkErrors> {
        let bytes = prefix_hex::decode::<Vec<u8>>(&hex_string).map_err(|e| {
            SdkErrors::PrefixError(format!("Failed to decode string to bytes {}", e))
        })?;
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        let signer = SigningKey::from_bytes(&array);
        Ok(Self(signer))
    }
    pub fn verifying_key_as_hex_string(&self) -> String {
        prefix_hex::encode(self.0.verifying_key().to_bytes())
    }

    pub fn signing_key_as_hex_string(&self) -> String {
        prefix_hex::encode(self.0.to_bytes())
    }
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let private_key = SigningKey::generate(&mut rng);
        ProverAccessKey(private_key)
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::*;
    fn generate_random_hex_string(length: usize) -> String {
        let mut rng = thread_rng();
        let chars: Vec<char> = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..16);
                "0123456789abcdef".chars().nth(idx).unwrap()
            })
            .collect();
        chars.into_iter().collect()
    }
    #[test]
    fn test_prover_access_key_with_random_inputs() {
        for _ in 0..1000 {
            // Generate a random hex string of different lengths
            let length = thread_rng().gen_range(0..64); // Random length up to 64 characters
            let hex_string = generate_random_hex_string(length);

            // Test from_hex_string with random input
            if let Ok(prover_access_key) = ProverAccessKey::from_hex_string(&hex_string) {
                // If we successfully created a ProverAccessKey, check the conversion methods
                let signing_key_hex = prover_access_key.signing_key_as_hex_string();

                // Recreate the ProverAccessKey from the signing key hex
                if let Ok(recreated_key) = ProverAccessKey::from_hex_string(&signing_key_hex) {
                    assert_eq!(
                        prover_access_key.signing_key_as_hex_string(),
                        recreated_key.signing_key_as_hex_string()
                    );
                    assert_eq!(
                        prover_access_key.verifying_key_as_hex_string(),
                        recreated_key.verifying_key_as_hex_string()
                    );
                }
            }

            // Generate a new key and test hex string conversion
            let generated_key = ProverAccessKey::generate();
            let signing_key_hex = generated_key.signing_key_as_hex_string();

            // Recreate the ProverAccessKey from the signing key hex
            if let Ok(recreated_key) = ProverAccessKey::from_hex_string(&signing_key_hex) {
                assert_eq!(
                    generated_key.signing_key_as_hex_string(),
                    recreated_key.signing_key_as_hex_string()
                );
                assert_eq!(
                    generated_key.verifying_key_as_hex_string(),
                    recreated_key.verifying_key_as_hex_string()
                );
            }
        }
    }
}
