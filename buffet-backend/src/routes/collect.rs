use axum::{Router, routing::post};
use crate::state::AppState;
use crate::handlers::collect::trigger_collection;

pub fn create_routes() -> Router<AppState> {
    Router::new().route("/api/collect", post(trigger_collection))
}
