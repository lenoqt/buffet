use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::Result;
use crate::models::{CreateUserDto, UpdateUserDto, User};
use crate::state::AppState;

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>> {
    let users = User::find_all(&state.db).await?;
    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>> {
    let user = User::find_by_id(&id, &state.db).await?;
    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<Json<User>> {
    let user = User::create(payload, &state.db).await?;
    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<User>> {
    let user = User::update(&id, payload, &state.db).await?;
    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>> {
    User::delete(&id, &state.db).await?;
    Ok(Json(()))
}