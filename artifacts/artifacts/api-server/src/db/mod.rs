use crate::models::*;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_credential_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Credential>> {
    sqlx::query_as!(
        Credential,
        r#"SELECT id, token_id, issuer_id, holder_address, credential_type, title,
                  metadata_uri, ipfs_hash, is_revoked, revoked_at, issued_at, created_at
           FROM credentials WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_credentials_by_holder(
    pool: &PgPool,
    holder: &str,
) -> sqlx::Result<Vec<Credential>> {
    sqlx::query_as!(
        Credential,
        r#"SELECT id, token_id, issuer_id, holder_address, credential_type, title,
                  metadata_uri, ipfs_hash, is_revoked, revoked_at, issued_at, created_at
           FROM credentials WHERE holder_address = $1 ORDER BY issued_at DESC"#,
        holder
    )
    .fetch_all(pool)
    .await
}

pub async fn create_credential(
    pool: &PgPool,
    issuer_id: Option<Uuid>,
    holder_address: &str,
    credential_type: &str,
    title: Option<&str>,
    metadata_uri: Option<&str>,
    ipfs_hash: Option<&str>,
) -> sqlx::Result<Credential> {
    sqlx::query_as!(
        Credential,
        r#"INSERT INTO credentials (issuer_id, holder_address, credential_type, title, metadata_uri, ipfs_hash)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, token_id, issuer_id, holder_address, credential_type, title,
                     metadata_uri, ipfs_hash, is_revoked, revoked_at, issued_at, created_at"#,
        issuer_id,
        holder_address,
        credential_type,
        title,
        metadata_uri,
        ipfs_hash,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_credential_token_id(
    pool: &PgPool,
    id: Uuid,
    token_id: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE credentials SET token_id = $1 WHERE id = $2",
        token_id,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn revoke_credential(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Credential>> {
    sqlx::query_as!(
        Credential,
        r#"UPDATE credentials SET is_revoked = true, revoked_at = NOW()
           WHERE id = $1 AND is_revoked = false
           RETURNING id, token_id, issuer_id, holder_address, credential_type, title,
                     metadata_uri, ipfs_hash, is_revoked, revoked_at, issued_at, created_at"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn log_transaction(
    pool: &PgPool,
    tx_hash: Option<&str>,
    credential_id: Option<Uuid>,
    action: &str,
    from_address: Option<&str>,
    status: &str,
) -> sqlx::Result<Transaction> {
    sqlx::query_as!(
        Transaction,
        r#"INSERT INTO transactions (tx_hash, credential_id, action, from_address, status)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, tx_hash, credential_id, action, from_address, status, created_at"#,
        tx_hash,
        credential_id,
        action,
        from_address,
        status,
    )
    .fetch_one(pool)
    .await
}

pub async fn log_verification(
    pool: &PgPool,
    credential_id: Option<Uuid>,
    verifier_address: Option<&str>,
    result: bool,
    message: Option<&str>,
) -> sqlx::Result<VerificationLog> {
    sqlx::query_as!(
        VerificationLog,
        r#"INSERT INTO verification_logs (credential_id, verifier_address, result, message)
           VALUES ($1, $2, $3, $4)
           RETURNING id, credential_id, verifier_address, result, message, created_at"#,
        credential_id,
        verifier_address,
        result,
        message,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_issuers(pool: &PgPool) -> sqlx::Result<Vec<Issuer>> {
    sqlx::query_as!(
        Issuer,
        r#"SELECT id, name, wallet_address, organization, is_active, created_at
           FROM issuers WHERE is_active = true ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_issuer_by_wallet(pool: &PgPool, wallet: &str) -> sqlx::Result<Option<Issuer>> {
    sqlx::query_as!(
        Issuer,
        r#"SELECT id, name, wallet_address, organization, is_active, created_at
           FROM issuers WHERE wallet_address = $1"#,
        wallet
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_issuer(
    pool: &PgPool,
    name: &str,
    wallet_address: &str,
    organization: Option<&str>,
) -> sqlx::Result<Issuer> {
    sqlx::query_as!(
        Issuer,
        r#"INSERT INTO issuers (name, wallet_address, organization)
           VALUES ($1, $2, $3)
           RETURNING id, name, wallet_address, organization, is_active, created_at"#,
        name,
        wallet_address,
        organization,
    )
    .fetch_one(pool)
    .await
}
