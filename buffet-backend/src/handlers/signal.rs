use crate::{error::Result, models::signal::Signal, state::AppState};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use std::collections::HashMap;

pub async fn list_signals(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Signal>>> {
    if let Some(strategy_id) = params.get("strategy_id") {
        Signal::find_by_strategy_id(strategy_id, &state.db).await.map(Json)
    } else {
        Signal::find_all(&state.db).await.map(Json)
    }
}

pub async fn get_signal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Signal>> {
    Signal::find_by_id(&id, &state.db).await.map(Json)
}
