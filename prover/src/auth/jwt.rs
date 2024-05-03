use crate::prove::ProveError;
use axum::{async_trait, extract::FromRequestParts, http::header::COOKIE, http::request::Parts};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("ENV_VAR_JWT_SECRET_KEY").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub fn encode_jwt(sub: &str, exp: usize) -> Result<String, ProveError> {
    let claims = Claims {
        sub: sub.to_owned(),
        exp,
    };
    encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|e| ProveError::InternalServerError(format!("JWT generation failed: {}", e)))
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ProveError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the 'Cookie' header
        let header_value = parts.headers.get(COOKIE).ok_or(ProveError::Auth(
            crate::prove::AuthError::MissingAuthorizationHeader,
        ))?;

        // Convert the header value to a string
        let cookie_str = header_value
            .to_str()
            .map_err(|_| ProveError::Auth(crate::prove::AuthError::InvalidToken))?;

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
        let token = cookies.get("jwt_token").ok_or(ProveError::Auth(
            crate::prove::AuthError::MissingAuthorizationHeader,
        ))?;

        let token_data = decode::<Claims>(token, &KEYS.decoding, &Validation::default())
            .map_err(|_| ProveError::Auth(crate::prove::AuthError::InvalidToken))?;

        Ok(token_data.claims)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}
impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sub: {}", self.sub)
    }
}
struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
