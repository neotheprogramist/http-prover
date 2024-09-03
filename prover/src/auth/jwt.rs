use std::{collections::HashMap, fmt::Display};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::COOKIE, request::Parts},
};
use ed25519_dalek::VerifyingKey;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{auth::auth_errors::AuthError, errors::ProverError, server::AppState};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ProverError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let store: AppState = AppState::from_ref(_state);

        // Extract the 'Cookie' header
        let cookie_str = parts
            .headers
            .get(COOKIE)
            .ok_or_else(|| {
                warn!("Missing 'Cookie' header in the request");
                ProverError::Auth(AuthError::MissingAuthorizationHeader)
            })?
            .to_str()
            .map_err(|_| ProverError::Auth(AuthError::InvalidToken))?;

        // Parse the cookie string into a HashMap
        let cookies: HashMap<_, _> = cookie_str
            .split(';')
            .filter_map(|cookie| {
                let mut parts = cookie.split('=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.trim(), value.trim())),
                    _ => None,
                }
            })
            .collect();

        // Extract the JWT token from the cookies
        let token = cookies
            .get("jwt_token")
            .ok_or(ProverError::Auth(AuthError::MissingAuthorizationHeader))?;

        let token_data = decode::<Claims>(
            token,
            &Keys::new(&store.jwt_secret_key.into_bytes()).decoding,
            &Validation::default(),
        )
        .map_err(|_| ProverError::Auth(AuthError::InvalidToken))?;

        Ok(token_data.claims)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub session_key: VerifyingKey,
}
impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sub: {}", self.sub)
    }
}
pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
pub fn encode_jwt(
    sub: &str,
    exp: usize,
    keys: Keys,
    session_key: VerifyingKey,
) -> Result<String, ProverError> {
    let claims = Claims {
        sub: sub.to_owned(),
        exp,
        session_key,
    };
    encode(&Header::default(), &claims, &keys.encoding)
        .map_err(|e| ProverError::InternalServerError(format!("JWT generation failed: {}", e)))
}
