use axum::{Router, http::Method};
use sqlx::{Pool, Sqlite};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

mod user;
mod item;
mod health;

pub fn create_router(pool: Pool<Sqlite>) -> Router {
    // Create application state
    let state = AppState::new(pool);

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Create router with all routes
    create_router_with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

// Used by the test harness to create a router with a specific state
pub fn create_test_router(state: AppState) -> Router {
    create_router_with_state(state)
}

// Common router creation logic
fn create_router_with_state(state: AppState) -> Router {
    Router::new()
        .merge(user::create_routes())
        .merge(item::create_routes())
        .merge(health::create_routes())
        .with_state(state)
}