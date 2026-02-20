use axum::{Router, http::Method};
use sqlx::{Pool, Postgres, Sqlite};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

mod health;
mod strategy;

use crate::actors::{DataCollectorActor, OrderExecutionActor, StrategyExecutorActor};
use kameo::actor::ActorRef;

pub fn create_router(
    db_pool: Pool<Sqlite>,
    tsdb_pool: Pool<Postgres>,
    collector_actor: ActorRef<DataCollectorActor>,
    executor_actor: ActorRef<StrategyExecutorActor>,
    execution_actor: ActorRef<OrderExecutionActor>,
) -> Router {
    // Create application state
    let state = AppState::new(
        db_pool,
        tsdb_pool,
        collector_actor,
        executor_actor,
        execution_actor,
    );

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
        .merge(strategy::create_routes())
        .merge(health::create_routes())
        .with_state(state)
}
