use crate::{handlers::strategy, state::AppState};
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/strategies", get(strategy::list_strategies))
        .route("/api/strategies", post(strategy::create_strategy))
        .route("/api/strategies/{id}", get(strategy::get_strategy))
        .route("/api/strategies/{id}", put(strategy::update_strategy))
        .route("/api/strategies/{id}", delete(strategy::delete_strategy))
}
