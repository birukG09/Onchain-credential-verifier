use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::OptionalAuth,
    blockchain::BlockchainClient,
    db,
    error::{AppError, AppResult},
    models::Credential,
    services::{self, compute_metadata_hash},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mint", post(mint))
        .route("/{id}", get(get_credential))
        .route("/verify", post(verify))
        .route("/revoke", post(revoke))
}

#[derive(Deserialize)]
pub struct MintRequest {
    pub holder_address: String,
    pub issuer_wallet: String,
    pub credential_type: String,
    pub title: Option<String>,
    pub metadata_uri: Option<String>,
    pub ipfs_hash: Option<String>,
}

#[derive(Serialize)]
pub struct MintResponse {
    pub credential: Credential,
    pub metadata_hash: Option<String>,
}

pub async fn mint(
    State(state): State<AppState>,
    _auth: OptionalAuth,
    Json(body): Json<MintRequest>,
) -> AppResult<Json<MintResponse>> {
    if body.holder_address.is_empty() {
        return Err(AppError::BadRequest("holder_address is required".to_string()));
    }
    if body.issuer_wallet.is_empty() {
        return Err(AppError::BadRequest("issuer_wallet is required".to_string()));
    }
    if body.credential_type.is_empty() {
        return Err(AppError::BadRequest("credential_type is required".to_string()));
    }

    let blockchain = BlockchainClient::new(&state.config.rpc_url, &state.config.contract_address);

    let credential = services::CredentialService::mint(
        &state.db,
        &body.issuer_wallet,
        &body.holder_address,
        &body.credential_type,
        body.title.as_deref(),
        body.metadata_uri.as_deref(),
        body.ipfs_hash.as_deref(),
        &blockchain,
    )
    .await?;

    let metadata_hash = body
        .metadata_uri
        .as_deref()
        .map(compute_metadata_hash);

    Ok(Json(MintResponse {
        credential,
        metadata_hash,
    }))
}

#[derive(Serialize)]
pub struct CredentialResponse {
    pub credential: Credential,
    pub issuer: Option<crate::models::Issuer>,
}

pub async fn get_credential(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CredentialResponse>> {
    let credential = db::get_credential_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Credential {} not found", id)))?;

    let issuer = if let Some(issuer_id) = credential.issuer_id {
        sqlx::query_as!(
            crate::models::Issuer,
            r#"SELECT id, name, wallet_address, organization, is_active, created_at
               FROM issuers WHERE id = $1"#,
            issuer_id
        )
        .fetch_optional(&state.db)
        .await?
    } else {
        None
    };

    Ok(Json(CredentialResponse { credential, issuer }))
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub credential_id: Uuid,
    pub verifier_address: Option<String>,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub message: String,
    pub credential: Credential,
}

pub async fn verify(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> AppResult<Json<VerifyResponse>> {
    let blockchain = BlockchainClient::new(&state.config.rpc_url, &state.config.contract_address);

    let (valid, message, credential) = services::CredentialService::verify(
        &state.db,
        body.credential_id,
        body.verifier_address.as_deref(),
        &blockchain,
    )
    .await?;

    Ok(Json(VerifyResponse {
        valid,
        message,
        credential,
    }))
}

#[derive(Deserialize)]
pub struct RevokeRequest {
    pub credential_id: Uuid,
    pub revoker_wallet: String,
}

#[derive(Serialize)]
pub struct RevokeResponse {
    pub success: bool,
    pub credential: Credential,
}

pub async fn revoke(
    State(state): State<AppState>,
    Json(body): Json<RevokeRequest>,
) -> AppResult<Json<RevokeResponse>> {
    if body.revoker_wallet.is_empty() {
        return Err(AppError::BadRequest(
            "revoker_wallet is required".to_string(),
        ));
    }

    let credential =
        services::CredentialService::revoke(&state.db, body.credential_id, &body.revoker_wallet)
            .await?;

    Ok(Json(RevokeResponse {
        success: true,
        credential,
    }))
}
