use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct JWTResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub jwt_token: String,
    pub expiration: u64,
    pub session_key: Option<VerifyingKey>,
}
#[derive(Serialize, Deserialize, Clone)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
