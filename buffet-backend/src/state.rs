use sqlx::{Pool, Postgres, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub tsdb: Pool<Postgres>,
}

impl AppState {
    pub fn new(db: Pool<Sqlite>, tsdb: Pool<Postgres>) -> Self {
        Self { db, tsdb }
    }
}
