use crate::{handlers::order, state::AppState};
use axum::{Router, routing::get};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/orders", get(order::list_orders))
        .route("/api/orders/{id}", get(order::get_order))
}
