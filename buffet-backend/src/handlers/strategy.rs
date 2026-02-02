use crate::{
    error::Result,
    models::strategy::{CreateStrategyDto, Strategy, UpdateStrategyDto},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

pub async fn list_strategies(State(state): State<AppState>) -> Result<Json<Vec<Strategy>>> {
    let strategies = Strategy::find_all(&state.db).await?;
    Ok(Json(strategies))
}

pub async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Strategy>> {
    let strategy = Strategy::find_by_id(&id, &state.db).await?;
    Ok(Json(strategy))
}

#[tracing::instrument(name = "Saving new strategy in the database", skip(state, dto))]
pub async fn create_strategy(
    State(state): State<AppState>,
    Json(dto): Json<CreateStrategyDto>,
) -> Result<(StatusCode, Json<Strategy>)> {
    let strategy = Strategy::create(dto, &state.db).await.map_err(|e| {
        tracing::error!("Failed to create strategy: {:?}", e);
        e
    })?;
    Ok((StatusCode::CREATED, Json(strategy)))
}

pub async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateStrategyDto>,
) -> Result<Json<Strategy>> {
    let strategy = Strategy::update(&id, dto, &state.db).await?;
    Ok(Json(strategy))
}

pub async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    Strategy::delete(&id, &state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
