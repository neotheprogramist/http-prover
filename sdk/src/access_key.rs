use common::bytes_to_hex_string;
use ed25519_dalek::SigningKey;
use prefix_hex::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ProverAccessKey(pub SigningKey);

impl ProverAccessKey {
    pub fn new(private_key: SigningKey) -> Self {
        ProverAccessKey(private_key)
    }

    pub fn from_hex_string(hex_string: &str) -> Result<Self, Error> {
        let bytes = prefix_hex::decode::<Vec<u8>>(&hex_string)?;
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        let signer = SigningKey::from_bytes(&array);
        Ok(ProverAccessKey(signer))
    }

    pub fn verifying_key_as_hex_string(&self) -> String {
        bytes_to_hex_string(&self.0.verifying_key().to_bytes())
    }

    pub fn signing_key_as_hex_string(&self) -> String {
        bytes_to_hex_string(&self.0.to_bytes())
    }

    /// Notice that this key has to be register by the prover operator first.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let private_key = SigningKey::generate(&mut rng);
        ProverAccessKey(private_key)
    }
}

impl Serialize for ProverAccessKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&bytes_to_hex_string(&self.0.to_bytes()))
    }
}

impl<'de> Deserialize<'de> for ProverAccessKey {
    fn deserialize<D>(deserializer: D) -> Result<ProverAccessKey, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        Self::from_hex_string(&hex_string).map_err(serde::de::Error::custom)
    }
}
