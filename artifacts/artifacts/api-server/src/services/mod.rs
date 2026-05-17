use crate::{
    blockchain::BlockchainClient,
    db,
    error::{AppError, AppResult},
    models::Credential,
};
use sha3::{Digest, Keccak256};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CredentialService;

impl CredentialService {
    pub async fn mint(
        pool: &PgPool,
        issuer_wallet: &str,
        holder_address: &str,
        credential_type: &str,
        title: Option<&str>,
        metadata_uri: Option<&str>,
        ipfs_hash: Option<&str>,
        _blockchain: &BlockchainClient,
    ) -> AppResult<Credential> {
        let issuer = db::get_issuer_by_wallet(pool, issuer_wallet).await?;
        let issuer_id = issuer.map(|i| i.id);

        let credential = db::create_credential(
            pool,
            issuer_id,
            holder_address,
            credential_type,
            title,
            metadata_uri,
            ipfs_hash,
        )
        .await?;

        db::log_transaction(
            pool,
            None,
            Some(credential.id),
            "mint",
            Some(issuer_wallet),
            "pending",
        )
        .await?;

        Ok(credential)
    }

    pub async fn verify(
        pool: &PgPool,
        credential_id: Uuid,
        verifier_address: Option<&str>,
        blockchain: &BlockchainClient,
    ) -> AppResult<(bool, String, Credential)> {
        let credential = db::get_credential_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Credential {} not found", credential_id))
            })?;

        if credential.is_revoked {
            db::log_verification(
                pool,
                Some(credential_id),
                verifier_address,
                false,
                Some("Credential has been revoked"),
            )
            .await?;
            return Ok((false, "Credential has been revoked".to_string(), credential));
        }

        let mut on_chain_valid = true;
        if blockchain.is_configured() {
            if let Some(token_id) = &credential.token_id {
                on_chain_valid = blockchain
                    .is_credential_valid(token_id)
                    .await
                    .unwrap_or(false);
            }
        }

        let result = on_chain_valid;
        let message = if result {
            "Credential is valid and authentic".to_string()
        } else {
            "Credential could not be verified on-chain".to_string()
        };

        db::log_verification(
            pool,
            Some(credential_id),
            verifier_address,
            result,
            Some(&message),
        )
        .await?;

        Ok((result, message, credential))
    }

    pub async fn revoke(
        pool: &PgPool,
        credential_id: Uuid,
        revoker_wallet: &str,
    ) -> AppResult<Credential> {
        let credential = db::revoke_credential(pool, credential_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound("Credential not found or already revoked".to_string())
            })?;

        db::log_transaction(
            pool,
            None,
            Some(credential_id),
            "revoke",
            Some(revoker_wallet),
            "confirmed",
        )
        .await?;

        Ok(credential)
    }
}

pub fn compute_metadata_hash(metadata_uri: &str) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(metadata_uri.as_bytes());
    format!("0x{}", hex::encode(hasher.finalize()))
}
