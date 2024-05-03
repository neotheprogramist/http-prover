use crate::prove::models::{
    GenerateNonceRequest, GenerateNonceResponse, JWTResponse, Nonce, ValidateSignatureRequest,
};
use crate::prove::ProveError;
use crate::server::AppState;
use axum::{
    extract::{Json, Query, State},
    http::{self, HeaderMap, HeaderValue},
    response::IntoResponse,
};
use jwt::encode_jwt;
use std::env;
use validation::verify_signature;

pub mod jwt;
pub mod validation;

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
    println!(" Generate key");

    if params.public_key.trim().is_empty() {
        println!("Missing public key");
        return Err(ProveError::MissingPublicKey);
    }

    let message_expiration_str = env::var("MESSAGE_EXPIRATION_TIME")
        .expect("MESSAGE_EXPIRATION_TIME environment variable not found!");
    let message_expiration_time: usize = message_expiration_str.parse::<usize>().unwrap();

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
    let message_expiration_str = env::var("SESSION_EXPIRATION_TIME")
        .expect("SESSION_EXPIRATION_TIME environment variable not found!");

    let session_expiration_time: usize = message_expiration_str.parse::<usize>().unwrap();

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

    let signature_valid = verify_signature(&payload.signature, &user_nonce, &payload.public_key);

    if !signature_valid {
        return Err(ProveError::Unauthorized("Invalid signature".to_string()));
    }

    let expiration = chrono::Utc::now() + chrono::Duration::seconds(session_expiration_time as i64);
    let token = encode_jwt(&payload.public_key, expiration.timestamp() as usize)
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
