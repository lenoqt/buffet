use axum::{Router, http::Method};
use sqlx::{Pool, Sqlite};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

mod user;
mod item;

pub fn create_router(pool: Pool<Sqlite>) -> Router {
    // Create application state
    let state = AppState::new(pool);

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Create router with all routes
    Router::new()
        .merge(user::create_routes())
        .merge(item::create_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}