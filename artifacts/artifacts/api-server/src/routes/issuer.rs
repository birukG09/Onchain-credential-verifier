use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    db,
    error::{AppError, AppResult},
    models::Issuer,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/register", post(register))
}

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Issuer>>> {
    let issuers = db::list_issuers(&state.db).await?;
    Ok(Json(issuers))
}

#[derive(Deserialize)]
pub struct RegisterIssuerRequest {
    pub name: String,
    pub wallet_address: String,
    pub organization: Option<String>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterIssuerRequest>,
) -> AppResult<Json<Issuer>> {
    if body.name.is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if body.wallet_address.is_empty() {
        return Err(AppError::BadRequest(
            "wallet_address is required".to_string(),
        ));
    }

    if !body.wallet_address.starts_with("0x") || body.wallet_address.len() != 42 {
        return Err(AppError::BadRequest(
            "wallet_address must be a valid Ethereum address".to_string(),
        ));
    }

    let existing = db::get_issuer_by_wallet(&state.db, &body.wallet_address).await?;
    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Issuer with this wallet address already exists".to_string(),
        ));
    }

    let issuer = db::create_issuer(
        &state.db,
        &body.name,
        &body.wallet_address,
        body.organization.as_deref(),
    )
    .await?;

    Ok(Json(issuer))
}
