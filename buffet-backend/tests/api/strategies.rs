use crate::helpers::spawn_app;
// use buffet_backend::models::strategy::StrategyType;

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
}

#[tokio::test]
async fn list_strategies_returns_all_stored_strategies() {
    // Arrange
    let app = spawn_app().await;

    // Create one strategy first
    app.api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&serde_json::json!({
            "name": "Strategy 1",
            "strategy_type": "Classical",
            "parameters": {}
        }))
        .send()
        .await
        .expect("Failed to create strategy");

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
    assert!(strategies.len() >= 1);
    assert_eq!(strategies[0]["name"], "Strategy 1");
}
