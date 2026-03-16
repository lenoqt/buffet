use axum::{Json, extract::State, http::StatusCode};
use serde_json::{json, Value};
use crate::state::AppState;

pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let sqlite_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let tsdb_ok = sqlx::query("SELECT 1").execute(&state.tsdb).await.is_ok();

    let status = if sqlite_ok && tsdb_ok { "ok" } else { "degraded" };
    let code = if sqlite_ok && tsdb_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(json!({
            "status": status,
            "sqlite": if sqlite_ok { "ok" } else { "unavailable" },
            "tsdb": if tsdb_ok { "ok" } else { "unavailable" }
        })),
    )
}
