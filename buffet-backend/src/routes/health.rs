use crate::{handlers::health, state::AppState};
use axum::{Router, routing::get};

pub fn create_routes() -> Router<AppState> {
    Router::new().route("/api/health", get(health::health_check))
}
