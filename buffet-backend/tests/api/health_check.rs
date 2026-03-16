use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let response = app
        .api_client
        .get(&format!("{}/api/health", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["sqlite"], "ok");
    assert_eq!(body["tsdb"], "ok");
}
