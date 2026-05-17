use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "service": "On-Chain Credential Verifier API",
        "database": if db_ok { "connected" } else { "error" },
        "version": env!("CARGO_PKG_VERSION")
    }))
}
