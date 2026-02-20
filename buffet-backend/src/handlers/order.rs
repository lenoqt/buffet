use crate::{error::Result, models::order::Order, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
};

pub async fn list_orders(State(state): State<AppState>) -> Result<Json<Vec<Order>>> {
    let orders = Order::find_all(&state.db).await?;
    Ok(Json(orders))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Order>> {
    let order = Order::find_by_id(&id, &state.db).await?;
    Ok(Json(order))
}
