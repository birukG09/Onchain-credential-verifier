use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub role: String,
    pub nonce: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Issuer {
    pub id: Uuid,
    pub name: String,
    pub wallet_address: String,
    pub organization: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Credential {
    pub id: Uuid,
    pub token_id: Option<String>,
    pub issuer_id: Option<Uuid>,
    pub holder_address: String,
    pub credential_type: String,
    pub title: Option<String>,
    pub metadata_uri: Option<String>,
    pub ipfs_hash: Option<String>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub issued_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub tx_hash: Option<String>,
    pub credential_id: Option<Uuid>,
    pub action: String,
    pub from_address: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct VerificationLog {
    pub id: Uuid,
    pub credential_id: Option<Uuid>,
    pub verifier_address: Option<String>,
    pub result: bool,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}
