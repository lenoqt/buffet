use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use tracing::info;

pub async fn setup_database(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Create database if it doesn't exist
    match Sqlite::database_exists(database_url).await {
        Ok(exists) => {
            if !exists {
                info!("Creating database {}", database_url);
                Sqlite::create_database(database_url).await?;
            }
        }
        Err(e) => {
            // Log the error but try to continue
            tracing::warn!("Error checking if database exists: {}", e);
            // Attempt to create database anyway
            if let Err(e) = Sqlite::create_database(database_url).await {
                tracing::warn!("Error creating database: {}", e);
                // Continue anyway, the connect call will fail if the database truly doesn't exist
            }
        }
    }

    // Connect to the database
    info!("Connecting to database {}", database_url);
    let pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    info!("Running database migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}