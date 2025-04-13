use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::handlers::item::{
    create_item, delete_item, get_item, get_items, get_user_items, update_item,
};
use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/items", get(get_items))
        .route("/api/items", post(create_item))
        .route("/api/items/{id}", get(get_item))
        .route("/api/items/{id}", put(update_item))
        .route("/api/items/{id}", delete(delete_item))
        .route("/api/users/{user_id}/items", get(get_user_items))
}