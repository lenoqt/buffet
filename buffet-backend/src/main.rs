use buffet_backend::{
    config, db, routes,
    telemetry::{get_subscriber, init_subscriber},
};
use kameo::actor::Spawn;
use kameo::mailbox;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry
    let subscriber = get_subscriber("buffet_backend".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Load configuration
    let config = config::Config::from_env().map_err(|e| {
        error!("Configuration error: {}", e);
        e
    })?;
    let addr = config.server_addr;

    // Set up database connections
    let db_pool = db::setup_database(&config.database_url).await?;
    let tsdb_pool = db::setup_tsdb(&config.tsdb_url).await?;

    // Initialize actor system
    info!("Initializing actor system");
    let storage_actor = buffet_backend::actors::TimeSeriesStorageActor::spawn_with_mailbox(
        buffet_backend::actors::TimeSeriesStorageActor::new(tsdb_pool.clone()),
        mailbox::bounded(config.actor.mailbox_size),
    );
    let execution_actor = buffet_backend::actors::OrderExecutionActor::spawn_with_mailbox(
        buffet_backend::actors::OrderExecutionActor::new(db_pool.clone()),
        mailbox::bounded(config.actor.mailbox_size),
    );
    let strategy_actor = buffet_backend::actors::StrategyExecutorActor::spawn_with_mailbox(
        buffet_backend::actors::StrategyExecutorActor::new(
            db_pool.clone(),
            execution_actor.clone(),
        ),
        mailbox::bounded(config.actor.mailbox_size),
    );
    let collector_actor = buffet_backend::actors::DataCollectorActor::spawn_with_mailbox(
        buffet_backend::actors::DataCollectorActor::new(
            storage_actor.clone(),
            strategy_actor.clone(),
        ),
        mailbox::bounded(config.actor.mailbox_size),
    );
    let backtest_actor = buffet_backend::actors::BacktestActor::spawn_with_mailbox(
        buffet_backend::actors::BacktestActor::new(db_pool.clone(), storage_actor.clone()),
        mailbox::bounded(config.actor.mailbox_size),
    );

    // Build our application with the database pools as state
    let app = routes::create_router(
        db_pool.clone(),
        tsdb_pool.clone(),
        collector_actor.clone(),
        strategy_actor.clone(),
        execution_actor.clone(),
        backtest_actor.clone(),
    );

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
