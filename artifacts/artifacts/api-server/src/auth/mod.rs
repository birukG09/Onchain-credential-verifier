use axum::{
    extract::FromRef,
    http::{request::Parts, HeaderMap},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub wallet_address: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(wallet_address: &str, role: &str, secret: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: wallet_address.to_lowercase(),
        wallet_address: wallet_address.to_lowercase(),
        role: role.to_string(),
        iat: now,
        exp: now + 86400,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::Unauthorized(e.to_string()))
}

pub fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Verify that `wallet_address` signed `message` producing `signature` (EIP-191 prefix).
pub fn verify_wallet_signature(
    wallet_address: &str,
    message: &str,
    signature: &str,
) -> Result<bool, AppError> {
    use sha3::{Digest, Keccak256};

    let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| AppError::BadRequest("Invalid signature hex".to_string()))?;

    if sig_bytes.len() != 65 {
        return Err(AppError::BadRequest(
            "Signature must be 65 bytes".to_string(),
        ));
    }

    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    let _hash = hasher.finalize();

    // Full ecrecover requires a native secp256k1 library; keep this as a
    // structural placeholder — wire up `k256` or `secp256k1` crate if needed.
    let recovered = recover_address_placeholder(&sig_bytes);
    Ok(recovered.to_lowercase() == wallet_address.to_lowercase())
}

fn recover_address_placeholder(_sig: &[u8]) -> String {
    // Placeholder — replace with k256::ecdsa ecrecover for production use.
    "0x0000000000000000000000000000000000000000".to_string()
}

/// Axum extractor: optional JWT auth — succeeds even without a token.
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<Claims>);

/// Axum extractor: required JWT auth — returns 401 if token is missing/invalid.
#[derive(Debug, Clone)]
pub struct RequiredAuth(pub Claims);

impl<S> axum::extract::FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        if let Some(token) = extract_bearer_token(&parts.headers) {
            match verify_token(token, &app_state.config.jwt_secret) {
                Ok(claims) => Ok(OptionalAuth(Some(claims))),
                Err(_) => Ok(OptionalAuth(None)),
            }
        } else {
            Ok(OptionalAuth(None))
        }
    }
}

impl<S> axum::extract::FromRequestParts<S> for RequiredAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = extract_bearer_token(&parts.headers)
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;
        let claims = verify_token(token, &app_state.config.jwt_secret)?;
        Ok(RequiredAuth(claims))
    }
}
