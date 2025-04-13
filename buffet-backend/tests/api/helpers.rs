use axum::Router;
use buffet_backend::{
    routes,
    state::AppState,
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, Pool, Sqlite, SqliteConnection, SqlitePool};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Ensure tests run sequentially
static TEST_MUTEX: Lazy<Arc<Mutex<()>>> = Lazy::new(|| Arc::new(Mutex::new(())));

pub struct TestApp {
    pub address: String,
    pub db_pool: Pool<Sqlite>,
    pub api_client: reqwest::Client,
}

// Create a test application with an isolated database
pub async fn spawn_app() -> TestApp {
    // Acquire mutex to run tests sequentially
    let _lock = TEST_MUTEX.lock().await;

    // Configure test database
    // let db_name = Uuid::new_v4().to_string();
    let db_url = format!("sqlite::memory:");

    // Initialize database
    let pool = setup_test_database(&db_url).await;

    // Create app and state
    let app_state = AppState::new(pool.clone());
    let app = routes::create_test_router(app_state);

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
        db_pool: pool,
        api_client: client,
    }
}

// Set up an isolated test database with migrations
async fn setup_test_database(db_url: &str) -> Pool<Sqlite> {
    // Create connection
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations on the connection
    for migration in std::fs::read_dir("./migrations").unwrap() {
        let migration = migration.unwrap();
        let path = migration.path();
        if path.is_file() && path.extension().unwrap_or_default() == "sql" {
            let sql = std::fs::read_to_string(path)
                .expect("Failed to read migration file");
            conn.execute(sql.as_str())
                .await
                .expect("Failed to execute migration");
        }
    }

    // Create a pool
    SqlitePool::connect(db_url)
        .await
        .expect("Failed to create test database pool")
}

// Testing utilities
impl TestApp {
    // Helper to create a test user
    pub async fn create_test_user(&self, username: &str, email: &str) -> String {
        let response = self.api_client
            .post(&format!("{}/api/users", &self.address))
            .json(&serde_json::json!({
                "username": username,
                "email": email,
                "password": "password123"
            }))
            .send()
            .await
            .expect("Failed to create test user");

        let user: serde_json::Value = response.json().await.unwrap();
        user["id"].as_str().unwrap().to_string()
    }

    // Helper to create a test item
    pub async fn create_test_item(&self, name: &str, user_id: &str) -> String {
        let response = self.api_client
            .post(&format!("{}/api/items", &self.address))
            .json(&serde_json::json!({
                "name": name,
                "description": "Test description",
                "user_id": user_id
            }))
            .send()
            .await
            .expect("Failed to create test item");

        let item: serde_json::Value = response.json().await.unwrap();
        item["id"].as_str().unwrap().to_string()
    }
}