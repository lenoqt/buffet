use crate::{handlers::position, state::AppState};
use axum::{Router, routing::get};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/positions", get(position::list_positions))
        .route("/api/positions/open", get(position::list_open_positions))
        .route("/api/positions/{id}", get(position::get_position))
}
