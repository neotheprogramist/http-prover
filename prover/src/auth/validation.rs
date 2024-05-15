use crate::auth::jwt::encode_jwt;
use crate::auth::jwt::Keys;
use crate::prove::errors::ProveError;
use crate::prove::models::{
    GenerateNonceRequest, GenerateNonceResponse, JWTResponse, Nonce, ValidateSignatureRequest,
};
use crate::server::AppState;
use axum::{
    extract::{Json, Query, State},
    http::{self, HeaderMap, HeaderValue},
    response::IntoResponse,
};
use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use std::collections::HashSet;
use tokio::{fs::File, io::AsyncReadExt};
pub const COOKIE_NAME: &str = "jwt_token";

/// Generates a nonce for a given public key and stores it in the application state.
///
/// # Parameters
///
/// - `state`: The application state containing a mutex-guarded HashMap to store nonces.
/// - `params`: The query parameters containing the public key for which nonce is generated.
///
/// # Returns
///
/// Returns a JSON response containing the generated nonce and its expiration time, or
/// an error if the public key is missing.
pub async fn generate_nonce(
    State(state): State<AppState>,
    Query(params): Query<GenerateNonceRequest>,
) -> Result<Json<GenerateNonceResponse>, ProveError> {
    if params.public_key.trim().is_empty() {
        return Err(ProveError::MissingPublicKey);
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("manifest_dir: {:?}", manifest_dir);
    let path = manifest_dir + ("/authorized_keys.json");
    println!("path: {:?}", path);
    is_public_key_authorized(&path, &params.public_key).await?;
    let message_expiration_time: usize = state.message_expiration_time;
    let nonce: Nonce = Nonce::new(32);
    let nonce_string = nonce.to_string();
    let mut nonces: std::sync::MutexGuard<'_, std::collections::HashMap<String, String>> =
        state.nonces.lock().unwrap();
    let formatted_key = params.public_key.trim().to_lowercase();

    nonces.insert(formatted_key.clone(), nonce_string);

    Ok(Json(GenerateNonceResponse {
        nonce,
        expiration: message_expiration_time,
    }))
}

/// Validates the signature provided in the request payload and generates a JWT token if the signature is valid.
///
/// # Parameters
///
/// - `state`: The application state containing nonce information stored in a mutex-guarded HashMap.
/// - `payload`: JSON payload containing the public key and signature to be validated.
///
/// # Returns
///
/// Returns a tuple containing HTTP headers and a JSON response with a JWT token and its expiration time if the signature is valid.
pub async fn validate_signature(
    State(state): State<AppState>,
    Json(payload): Json<ValidateSignatureRequest>,
) -> Result<impl IntoResponse, ProveError> {
    is_public_key_authorized("prover/authorized_keys.json", &payload.public_key).await?;

    let session_expiration_time: usize = state.session_expiration_time;

    let nonces = state
        .nonces
        .lock()
        .map_err(|_| ProveError::InternalServerError("Failed to lock state".to_string()))?;

    let user_nonce = nonces.get(&payload.public_key).ok_or_else(|| {
        ProveError::NotFound(format!(
            "Nonce not found for the provided public key: {}",
            &payload.public_key
        ))
    })?;

    let signature_valid = verify_signature(&payload.signature, user_nonce, &payload.public_key);

    if !signature_valid {
        return Err(ProveError::Unauthorized("Invalid signature".to_string()));
    }

    let expiration = chrono::Utc::now() + chrono::Duration::seconds(session_expiration_time as i64);
    let token = encode_jwt(
        &payload.public_key,
        expiration.timestamp() as usize,
        Keys::new(state.jwt_secret_key.clone().as_bytes()),
    )
    .map_err(|_| ProveError::InternalServerError("JWT generation failed".to_string()))?;
    let cookie_value = format!(
        "{}={}; HttpOnly; Secure; Path=/; Max-Age={}",
        COOKIE_NAME, token, session_expiration_time
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie_value).map_err(|_| {
            ProveError::InternalServerError("Failed to set cookie header".to_string())
        })?,
    );

    Ok((
        headers,
        Json(JWTResponse {
            jwt_token: token,
            expiration: session_expiration_time,
        }),
    ))
}

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

    let mut public_key_array = [0u8; 32];
    public_key_array.copy_from_slice(&public_key_bytes[..32]); // Copy the first 32 bytes

    let public_key = match VerifyingKey::from_bytes(&public_key_array) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    public_key
        .verify_strict(nonce.as_bytes(), signature)
        .is_ok()
}

/// Reads the authorized keys from the `authorized_keys.json` file and returns them as a HashSet.
///
/// # Returns
///
/// Returns a HashSet containing the authorized keys, or an error if reading the file fails.
pub async fn is_public_key_authorized(path: &str, public_key: &str) -> Result<(), ProveError> {
    let formatted_key = public_key.trim().to_lowercase();

    // Read the authorized_keys.json file
    let mut file = File::open(path).await.map_err(ProveError::FileReadError)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .await
        .map_err(ProveError::FileReadError)?;

    let authorized_keys: HashSet<String> = serde_json::from_str::<Vec<String>>(&contents)
        .map_err(|_| ProveError::JsonParsingFailed("authorized_keys.json".to_string()))?
        .into_iter()
        .collect();

    if !authorized_keys.contains(&formatted_key) {
        return Err(ProveError::UnauthorizedPublicKey);
    }
    Ok(())
}
