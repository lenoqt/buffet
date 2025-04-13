use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

impl AppState {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }
}