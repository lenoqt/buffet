use crate::{handlers::signal, state::AppState};
use axum::{Router, routing::get};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/signals", get(signal::list_signals))
        .route("/api/signals/{id}", get(signal::get_signal))
}
