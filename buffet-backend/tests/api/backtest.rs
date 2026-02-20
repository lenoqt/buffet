use crate::helpers::spawn_app;
use buffet_backend::models::backtest::{Backtest, BacktestTrade, CreateBacktestDto};
use buffet_backend::models::market_data::OHLCV;
use buffet_backend::models::strategy::{CreateStrategyDto, Strategy, StrategyType};
use buffet_backend::tsdb::TimescaleDb;
use chrono::{Duration, Utc};
use serde_json::json;

#[tokio::test]
async fn test_backtest_execution_flow() {
    let app = spawn_app().await;

    // 1. Create a strategy
    let strategy_dto = CreateStrategyDto {
        name: "Test MA Crossover".to_string(),
        strategy_type: StrategyType::Classical,
        parameters: json!({
            "fast_period": 2,
            "slow_period": 4
        }),
    };

    let response = app
        .api_client
        .post(&format!("{}/api/strategies", &app.address))
        .json(&strategy_dto)
        .send()
        .await
        .expect("Failed to create strategy");

    assert_eq!(response.status(), 201);
    let strategy: Strategy = response.json().await.expect("Failed to parse strategy");

    // 2. Insert mock OHLCV data
    let symbol = "TEST_BT".to_string();
    let now = Utc::now();
    let mut data = Vec::new();

    // Create a price trend to trigger Buys and Sells
    // Prices: 10, 11, 12, 11, 10, 9, 8, 7, 8, 9, 10, 11, 12
    let prices = vec![
        10.0, 11.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ];
    for (i, &price) in prices.iter().enumerate() {
        data.push(OHLCV {
            timestamp: now - Duration::minutes((prices.len() - i) as i64),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 100.0,
        });
    }

    let tsdb = TimescaleDb::new(app.tsdb_pool.clone());
    tsdb.insert_ohlcv(&symbol, "crypto", &data)
        .await
        .expect("Failed to insert mock data");

    // 3. Run backtest
    let backtest_dto = CreateBacktestDto {
        strategy_id: strategy.id,
        symbol: symbol.clone(),
        start_time: now - Duration::hours(1),
        end_time: now + Duration::hours(1),
        initial_balance: 1000.0,
    };

    let response = app
        .api_client
        .post(&format!("{}/api/backtests", &app.address))
        .json(&backtest_dto)
        .send()
        .await
        .expect("Failed to run backtest");

    assert_eq!(response.status(), 202);
    let backtest_res: Backtest = response.json().await.expect("Failed to parse backtest");

    // 4. Poll for completion
    let mut completed_backtest = None;
    for _ in 0..20 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let response = app
            .api_client
            .get(&format!(
                "{}/api/backtests/{}",
                &app.address, backtest_res.id
            ))
            .send()
            .await
            .expect("Failed to get backtest status");

        let backtest: Backtest = response.json().await.expect("Failed to parse backtest");
        if backtest.status == "completed" {
            completed_backtest = Some(backtest);
            break;
        }
        if backtest.status == "failed" {
            panic!("Backtest failed: {:?}", backtest.error_message);
        }
    }

    let b = completed_backtest.expect("Backtest timed out");
    assert!(b.final_balance.is_some());
    assert!(b.total_return.is_some());
    assert!(b.sharpe_ratio.is_some());
    assert!(b.max_drawdown.is_some());

    // 5. Verify trades
    let response = app
        .api_client
        .get(&format!("{}/api/backtests/{}/trades", &app.address, b.id))
        .send()
        .await
        .expect("Failed to get trades");

    let trades: Vec<BacktestTrade> = response.json().await.expect("Failed to parse trades");
    assert!(trades.len() > 0);

    // Cleanup - not strictly necessary as we use isolated DBs
}
