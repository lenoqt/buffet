use crate::{handlers::backtest, state::AppState};
use axum::{
    Router,
    routing::{get, post},
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/backtests", get(backtest::list_backtests))
        .route("/api/backtests", post(backtest::run_backtest))
        .route("/api/backtests/{id}", get(backtest::get_backtest))
        .route(
            "/api/backtests/{id}/trades",
            get(backtest::get_backtest_trades),
        )
}
