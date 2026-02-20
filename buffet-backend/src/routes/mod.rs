use axum::Router;
use axum::http::{Method, header};
use sqlx::{Pool, Postgres, Sqlite};
use tower_http::cors::{Any, CorsLayer};

use crate::actors::{
    BacktestActor, DataCollectorActor, OrderExecutionActor, StrategyExecutorActor,
};
use crate::state::AppState;
use kameo::actor::ActorRef;

mod backtest;
mod health;
mod order;
mod position;
mod strategy;

pub fn create_router(
    db_pool: Pool<Sqlite>,
    tsdb_pool: Pool<Postgres>,
    collector_actor: ActorRef<DataCollectorActor>,
    executor_actor: ActorRef<StrategyExecutorActor>,
    execution_actor: ActorRef<OrderExecutionActor>,
    backtest_actor: ActorRef<BacktestActor>,
) -> Router {
    // Create application state
    let state = AppState::new(
        db_pool,
        tsdb_pool,
        collector_actor,
        executor_actor,
        execution_actor,
        backtest_actor,
    );

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers([header::CONTENT_TYPE]);

    // Build our application with the state
    create_router_with_state(state).layer(cors)
}

fn create_router_with_state(state: AppState) -> Router {
    Router::new()
        .merge(strategy::create_routes())
        .merge(order::create_routes())
        .merge(position::create_routes())
        .merge(backtest::create_routes())
        .merge(health::create_routes())
        .with_state(state)
}

#[cfg(test)]
pub fn create_test_router(state: AppState) -> Router {
    create_router_with_state(state)
}
