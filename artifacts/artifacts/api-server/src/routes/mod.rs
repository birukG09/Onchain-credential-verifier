use axum::{routing::get, Router};

use crate::AppState;

pub mod credentials;
pub mod health;
pub mod issuer;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::handler))
        .nest("/credentials", credentials::router())
        .nest("/issuer", issuer::router())
        .with_state(state)
}
