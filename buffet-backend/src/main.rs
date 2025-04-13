mod config;
mod db;
mod error;
mod handlers;
mod models;
mod routes;
mod state;

use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = config::Config::from_env()
        .map_err(|e| {
            error!("Configuration error: {}", e);
            e
        })?;
    let addr = config.server_addr;

    // Set up database connection
    let pool = db::setup_database(&config.database_url).await?;

    // Build our application with the database pool as state
    let app = routes::create_router(pool);

    // Start server
    info!("Starting server at {}", addr);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind server to address {}: {}", addr, e))?;

    // Start server with the listener
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}