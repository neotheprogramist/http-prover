use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAuthorizedRequest {
    pub signature: Signature,
    pub authority: VerifyingKey,
    pub new_key: VerifyingKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateSignatureRequest {
    pub signature: Signature,
    pub public_key: VerifyingKey,
}
