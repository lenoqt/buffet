use crate::{error::Result, models::position::Position, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
};

pub async fn list_positions(State(state): State<AppState>) -> Result<Json<Vec<Position>>> {
    let positions = Position::find_all(&state.db).await?;
    Ok(Json(positions))
}

pub async fn list_open_positions(State(state): State<AppState>) -> Result<Json<Vec<Position>>> {
    let positions = Position::find_open(&state.db).await?;
    Ok(Json(positions))
}

pub async fn get_position(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Position>> {
    let position = Position::find_by_id(&id, &state.db).await?;
    Ok(Json(position))
}
