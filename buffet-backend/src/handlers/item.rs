use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::Result;
use crate::models::{CreateItemDto, Item, UpdateItemDto};
use crate::state::AppState;

pub async fn get_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<Item>>> {
    let items = Item::find_all(&state.db).await?;
    Ok(Json(items))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Item>> {
    let item = Item::find_by_id(&id, &state.db).await?;
    Ok(Json(item))
}

pub async fn get_user_items(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Item>>> {
    let items = Item::find_by_user_id(&user_id, &state.db).await?;
    Ok(Json(items))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemDto>,
) -> Result<Json<Item>> {
    let item = Item::create(payload, &state.db).await?;
    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateItemDto>,
) -> Result<Json<Item>> {
    let item = Item::update(&id, payload, &state.db).await?;
    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>> {
    Item::delete(&id, &state.db).await?;
    Ok(Json(()))
}