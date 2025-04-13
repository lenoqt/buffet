use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::handlers::user::{
    create_user, delete_user, get_user, get_users, update_user,
};
use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/users", get(get_users))
        .route("/api/users", post(create_user))
        .route("/api/users/{id}", get(get_user))
        .route("/api/users/{id}", put(update_user))
        .route("/api/users/{id}", delete(delete_user))
}