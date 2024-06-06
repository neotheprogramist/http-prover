use crate::cert::types::base64;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use josekit::{
    jwk::alg::ec::EcKeyPair,
    jws::{JwsHeader, ES256},
    jwt:: JwtPayload,
};

use serde_json::json;

pub fn create_jws(
    nonce: String,
    payload: JwtPayload,
    url: String,
    ec_key_pair: EcKeyPair,
    kid: Option<String>,
) -> Result<String, josekit::JoseError> {
    // You would typically load your ECDSA P-256 key from secure storage or configuration
    // Convert key to JWK format for including in the protected header
    let mut header = JwsHeader::new();
    if kid.is_some() {
        let value = kid.unwrap();
        header.set_key_id(value); // Set the Key ID
    }else {
        let jwk = ec_key_pair.to_jwk_public_key();
        header.set_jwk(jwk); // Set the JWK
    }
    header.set_algorithm("ES256".to_string());

    //decoding nonce
    let nonce = URL_SAFE_NO_PAD.decode(nonce.as_bytes()).unwrap();
    header.set_nonce(nonce.clone());
    header.set_url(url);

    let encoded_header = base64(header.as_ref()).unwrap();
    let encoded_payload = base64(&payload.as_ref()).unwrap();
    let signer = ES256.signer_from_pem(ec_key_pair.to_pem_private_key())?;
    //Create and sign the JWT
    let signature = signer.sign(format!("{}.{}", &encoded_header, &encoded_payload).as_bytes())?;
    let encoded_signature = BASE64_URL_SAFE_NO_PAD.encode(signature);
    Ok(json!({
        "protected": encoded_header,
        "payload": encoded_payload,
        "signature": encoded_signature
    })
    .to_string())
}
