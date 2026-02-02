use crate::helpers::spawn_app;
use serde_json::json;

#[tokio::test]
async fn create_user_works() {
    let app = spawn_app().await;
    let payload = json!({"username": "testuser", "email": "test@test.com", "password": "testpassword"});

    let response = app.api_client
        .post(&format!("{}/api/users", app.address))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success(), "{}", format!("Failed to create user during test {:?}", response));
}