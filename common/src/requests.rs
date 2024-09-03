use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateSignatureRequest {
    pub signature: Signature,
    pub message: Message,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub session_key: VerifyingKey,
    pub nonce: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateNonceRequest {
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddKeyRequest {
    pub signature: Signature,
    pub authority: VerifyingKey,
    pub new_key: VerifyingKey,
}
