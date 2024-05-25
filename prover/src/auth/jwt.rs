use crate::prove::errors::{AuthError, ProveError};
use crate::server::AppState;
use axum::extract::FromRef;
use axum::{async_trait, extract::FromRequestParts, http::header::COOKIE, http::request::Parts};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use tracing::warn;

pub fn encode_jwt(sub: &str, exp: usize, keys: Keys) -> Result<String, ProveError> {
    let claims = Claims {
        sub: sub.to_owned(),
        exp,
    };
    encode(&Header::default(), &claims, &keys.encoding)
        .map_err(|e| ProveError::InternalServerError(format!("JWT generation failed: {}", e)))
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ProveError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let store: AppState = AppState::from_ref(_state);

        // Extract the 'Cookie' header
        let header_value = parts.headers.get(COOKIE).ok_or_else(|| {
            warn!("Missing 'Cookie' header in the request");
            ProveError::Auth(AuthError::MissingAuthorizationHeader)
        })?;
        // Convert the header value to a string
        let cookie_str = header_value
            .to_str()
            .map_err(|_| ProveError::Auth(AuthError::InvalidToken))?;

        // Parse the cookie string into a HashMap
        let cookies: HashMap<_, _> = cookie_str
            .split(';')
            .filter_map(|s| {
                let mut parts = s.split('=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.trim(), value.trim())),
                    _ => None,
                }
            })
            .collect();

        // Extract the JWT token from the cookies
        let token = cookies
            .get("jwt_token")
            .ok_or(ProveError::Auth(AuthError::MissingAuthorizationHeader))?;

        let token_data = decode::<Claims>(
            token,
            &Keys::new(&store.jwt_secret_key.into_bytes()).decoding,
            &Validation::default(),
        )
        .map_err(|_| ProveError::Auth(AuthError::InvalidToken))?;

        Ok(token_data.claims)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
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
