use crate::helpers::spawn_app;

#[tokio::test]
async fn create_strategy_returns_201_and_valid_json() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "Moving Average Cross",
            "strategy_type": "Classical",
            "parameters": {
                "fast_period": 10,
                "slow_period": 20
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Moving Average Cross");
    assert_eq!(body["strategy_type"], "classical");
    assert_eq!(body["status"], "inactive");
}

#[tokio::test]
async fn list_strategies_returns_all_stored_strategies() {
    // Arrange
    let app = spawn_app().await;

    // Create a strategy with valid Classical parameters
    let create_response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "List Test Strategy",
            "strategy_type": "Classical",
            "parameters": {
                "fast_period": 5,
                "slow_period": 15
            }
        }))
        .send()
        .await
        .expect("Failed to create strategy");

    assert_eq!(create_response.status().as_u16(), 201);

    // Act
    let response = app
        .api_client
        .get(&format!("{}/api/strategies", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let strategies: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON");
    assert!(!strategies.is_empty());

    // Find our strategy by name rather than relying on ordering
    let found = strategies
        .iter()
        .find(|s| s["name"] == "List Test Strategy");
    assert!(found.is_some(), "Expected to find 'List Test Strategy' in the list");

    let s = found.unwrap();
    assert_eq!(s["strategy_type"], "classical");
    assert_eq!(s["status"], "inactive");
}

#[tokio::test]
async fn create_strategy_with_invalid_parameters_returns_400() {
    let app = spawn_app().await;

    // fast_period >= slow_period should fail
    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "Bad Params",
            "strategy_type": "Classical",
            "parameters": {
                "fast_period": 20,
                "slow_period": 10
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_strategy_missing_required_parameters_returns_400() {
    let app = spawn_app().await;

    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "Missing Params",
            "strategy_type": "Classical",
            "parameters": {}
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn activate_and_deactivate_strategy() {
    let app = spawn_app().await;

    // Create a strategy
    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "Lifecycle Strategy",
            "strategy_type": "Classical",
            "parameters": {
                "fast_period": 5,
                "slow_period": 20
            }
        }))
        .send()
        .await
        .expect("Failed to create strategy");

    assert_eq!(response.status().as_u16(), 201);
    let strategy: serde_json::Value = response.json().await.unwrap();
    let id = strategy["id"].as_str().unwrap();
    assert_eq!(strategy["status"], "inactive");

    // Activate
    let response = app
        .api_client
        .put(&format!("{}/api/strategies/{}/activate", &app.address, id))
        .send()
        .await
        .expect("Failed to activate strategy");

    assert_eq!(response.status().as_u16(), 200);
    let activated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(activated["status"], "active");

    // Verify it appears in the active list
    let response = app
        .api_client
        .get(&format!("{}/api/strategies?status=active", &app.address))
        .send()
        .await
        .expect("Failed to list active strategies");

    assert_eq!(response.status().as_u16(), 200);
    let actives: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(actives.iter().any(|s| s["id"] == id));

    // Deactivate
    let response = app
        .api_client
        .put(&format!(
            "{}/api/strategies/{}/deactivate",
            &app.address, id
        ))
        .send()
        .await
        .expect("Failed to deactivate strategy");

    assert_eq!(response.status().as_u16(), 200);
    let deactivated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(deactivated["status"], "inactive");
}

#[tokio::test]
async fn get_strategy_by_id_returns_full_details() {
    let app = spawn_app().await;

    // Create
    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "Detail Strategy",
            "strategy_type": "Classical",
            "parameters": {
                "fast_period": 10,
                "slow_period": 30
            },
            "symbols": ["AAPL", "MSFT"]
        }))
        .send()
        .await
        .expect("Failed to create");

    assert_eq!(response.status().as_u16(), 201);
    let created: serde_json::Value = response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    // Fetch by ID
    let response = app
        .api_client
        .get(&format!("{}/api/strategies/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to get strategy");

    assert_eq!(response.status().as_u16(), 200);
    let fetched: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["name"], "Detail Strategy");
    assert_eq!(fetched["strategy_type"], "classical");
    assert_eq!(fetched["status"], "inactive");
}

#[tokio::test]
async fn delete_strategy_returns_204() {
    let app = spawn_app().await;

    // Create
    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "To Delete",
            "strategy_type": "Classical",
            "parameters": {
                "fast_period": 3,
                "slow_period": 7
            }
        }))
        .send()
        .await
        .expect("Failed to create");

    let created: serde_json::Value = response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    // Delete
    let response = app
        .api_client
        .delete(&format!("{}/api/strategies/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 204);

    // Verify gone
    let response = app
        .api_client
        .get(&format!("{}/api/strategies/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to get");

    assert_eq!(response.status().as_u16(), 404);
}
