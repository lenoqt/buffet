use sqlx::{PgPool, Pool, Postgres, Sqlite, SqlitePool, migrate::MigrateDatabase};
use tracing::info;

/// Set up the SQLite database for metadata
pub async fn setup_database(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await? {
        info!("Creating SQLite database {}", database_url);
        Sqlite::create_database(database_url).await?;
    }

    // Connect to the database
    info!("Connecting to SQLite database {}", database_url);
    let pool = SqlitePool::connect(database_url).await?;

    // Enable foreign keys and WAL mode for reliability and performance
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;

    // Run migrations
    info!("Running SQLite migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Set up the PostgreSQL/TimescaleDB for time-series data
pub async fn setup_tsdb(tsdb_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    // Note: For Postgres, we usually expect the DB to exist or we use a separate admin connection to create it.
    // For now, we connect directly.
    info!("Connecting to TimescaleDB at {}", tsdb_url);
    let pool = PgPool::connect(tsdb_url).await?;

    Ok(pool)
}

/// Initialize TimescaleDB tables and hypertables via the TimescaleDb abstraction
pub async fn setup_tsdb_tables(pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let tsdb = crate::tsdb::TimescaleDb::new(pool.clone());
    tsdb.setup()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to setup TSDB tables: {}", e))
}
