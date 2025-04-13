use axum::{
    routing::get,
    Router,
};

use crate::state::AppState;

pub async fn health_check() -> &'static str {
    "OK"
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health_check))
}