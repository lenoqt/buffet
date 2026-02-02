use buffet_backend::{
    routes,
    state::AppState,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{PgPool, SqlitePool};
use std::sync::Arc;
use tokio::sync::Mutex;

// Ensure tests run sequentially if they share resources
static TEST_MUTEX: Lazy<Arc<Mutex<()>>> = Lazy::new(|| Arc::new(Mutex::new(())));

// Initialize telemetry once for all tests
static TRACING: Lazy<()> = Lazy::new(|| {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
    let subscriber_name = "test".into();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, filter, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, filter, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub api_client: reqwest::Client,
}

// Create a test application with isolated databases
pub async fn spawn_app() -> TestApp {
    // Initialize tracing
    Lazy::force(&TRACING);

    // Acquire mutex to run tests sequentially if needed
    let _lock = TEST_MUTEX.lock().await;

    // Configure test databases
    let db_url = "sqlite::memory:".to_string();

    // In a real scenario, we might want a test-specific Postgres DB
    // For now, we use the one from env or a default test one
    let tsdb_url = std::env::var("TSDB_TEST_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/buffet_test".to_string());

    // Initialize SQLite database
    let db_pool = setup_test_sqlite(&db_url).await;

    // Initialize TSDB (Postgres)
    // First connect to default postgres to create our test db if needed
    let admin_url = tsdb_url.replace("/buffet_test", "/postgres");
    let admin_pool = PgPool::connect(&admin_url)
        .await
        .expect("Failed to connect to Postgres admin");

    // Create database if not exists
    let exists: (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'buffet_test')")
            .fetch_one(&admin_pool)
            .await
            .unwrap_or((false,));

    if !exists.0 {
        sqlx::query("CREATE DATABASE buffet_test")
            .execute(&admin_pool)
            .await
            .expect("Failed to create buffet_test database");
    }
    admin_pool.close().await;

    let tsdb_pool = PgPool::connect(&tsdb_url)
        .await
        .expect("Failed to connect to test TSDB");

    // Create app and state
    let _app_state = AppState::new(db_pool.clone(), tsdb_pool.clone());
    let app = routes::create_router(db_pool.clone(), tsdb_pool.clone());

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server_address = format!("http://127.0.0.1:{}", port);

    // Spawn the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Failed to run test server");
    });

    // Configure the client
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestApp {
        address: server_address,
        api_client: client,
    }
}

// Set up an isolated test SQLite database with migrations
async fn setup_test_sqlite(db_url: &str) -> SqlitePool {
    let pool = SqlitePool::connect(db_url)
        .await
        .expect("Failed to create test database pool");

    // Run migrations using sqlx
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

impl TestApp {}
