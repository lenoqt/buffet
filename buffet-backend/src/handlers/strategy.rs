use crate::{
    actors::messages::{RegisterStrategy, UnregisterStrategy},
    error::{AppError, Result},
    models::strategy::{
        validate_parameters, CreateStrategyDto, Strategy, StrategyStatus, UpdateStrategyDto,
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use std::collections::HashMap;

pub async fn list_strategies(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Strategy>>> {
    let strategies = match params.get("status") {
        Some(status) => Strategy::find_by_status(status, &state.db).await?,
        None => Strategy::find_all(&state.db).await?,
    };
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
    // Validate parameters before persisting
    validate_parameters(&dto.strategy_type, &dto.parameters)
        .map_err(AppError::BadRequest)?;

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
    // If new parameters are supplied, validate them against the strategy's current type
    if let Some(ref params) = dto.parameters {
        // Load existing strategy to get its type for validation
        let existing = Strategy::find_by_id(&id, &state.db).await?;
        let strategy_type = existing
            .strategy_type
            .parse::<crate::models::strategy::StrategyType>()
            .map_err(|e| AppError::BadRequest(format!("Invalid strategy type stored: {}", e)))?;

        validate_parameters(&strategy_type, params).map_err(AppError::BadRequest)?;
    }

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

pub async fn activate_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Strategy>> {
    let strategy = Strategy::set_status(&id, StrategyStatus::Active, &state.db).await?;

    // Notify the strategy executor to register this strategy at runtime
    let _ = state
        .executor
        .tell(RegisterStrategy {
            strategy_id: strategy.id.clone(),
            strategy_type: strategy.strategy_type.clone(),
            parameters: strategy.parameters.clone(),
        })
        .send()
        .await;

    Ok(Json(strategy))
}

pub async fn deactivate_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Strategy>> {
    let strategy = Strategy::set_status(&id, StrategyStatus::Inactive, &state.db).await?;

    // Notify the strategy executor to unregister this strategy at runtime
    let _ = state
        .executor
        .tell(UnregisterStrategy {
            strategy_id: strategy.id.clone(),
        })
        .send()
        .await;

    Ok(Json(strategy))
}
