use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct JWTResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub jwt_token: String,
    pub expiration: u64,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestCreateContract {
    pub address: String,
    pub quantity: u64,
    pub url_request_quote: String,
    pub url_accept_contract: String,
}
