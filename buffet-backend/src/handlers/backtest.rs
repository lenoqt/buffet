use crate::{
    actors::messages::RunBacktest,
    error::Result,
    models::backtest::{Backtest, BacktestTrade, CreateBacktestDto},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

pub async fn list_backtests(State(state): State<AppState>) -> Result<Json<Vec<Backtest>>> {
    let backtests = Backtest::find_all(&state.db).await?;
    Ok(Json(backtests))
}

pub async fn get_backtest(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Backtest>> {
    let backtest = Backtest::find_by_id(&id, &state.db).await?;
    Ok(Json(backtest))
}

pub async fn run_backtest(
    State(state): State<AppState>,
    Json(dto): Json<CreateBacktestDto>,
) -> Result<(StatusCode, Json<Backtest>)> {
    // 1. Create backtest record
    let backtest = Backtest::create(dto, &state.db).await?;

    // 2. Trigger backtest actor (fire-and-forget)
    let _ = state
        .backtest
        .tell(RunBacktest {
            backtest_id: backtest.id.clone(),
        })
        .send()
        .await;

    Ok((StatusCode::ACCEPTED, Json(backtest)))
}

pub async fn get_backtest_trades(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<BacktestTrade>>> {
    let trades = BacktestTrade::find_by_backtest(&id, &state.db).await?;
    Ok(Json(trades))
}
