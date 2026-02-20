use crate::helpers::spawn_app;
use buffet_backend::actors::messages::{MarketDataUpdate, SignalType};
use buffet_backend::actors::strategy::StrategyLogic;
use buffet_backend::actors::{OrderExecutionActor, StrategyExecutorActor};
use buffet_backend::models::order::OrderStatus;
use kameo::actor::Spawn;
use kameo::mailbox;

struct MockStrategy {
    should_signal: bool,
}

impl StrategyLogic for MockStrategy {
    fn update(&mut self, _data: &buffet_backend::models::market_data::OHLCV) -> Option<SignalType> {
        if self.should_signal {
            Some(SignalType::Buy)
        } else {
            None
        }
    }
}

#[tokio::test]
async fn test_order_creation_from_signal() {
    // 1. Spawn app
    let app = spawn_app().await;

    // 2. Setup OrderExecutionActor
    let execution_actor = OrderExecutionActor::spawn_with_mailbox(
        OrderExecutionActor::new(app.db_pool.clone()),
        mailbox::bounded(10),
    );

    // 3. Setup StrategyExecutorActor with execution actor
    let mut executor = StrategyExecutorActor::new(app.db_pool.clone(), execution_actor.clone());
    executor.register_strategy(
        "mock_order_strategy".to_string(),
        Box::new(MockStrategy {
            should_signal: true,
        }),
    );

    let strategy_actor_ref =
        StrategyExecutorActor::spawn_with_mailbox(executor, mailbox::bounded(10));

    // 4. Send MarketDataUpdate to trigger signal and order
    let candle = buffet_backend::models::market_data::OHLCV {
        timestamp: chrono::Utc::now(),
        open: 100.0,
        high: 100.0,
        low: 100.0,
        close: 100.0,
        volume: 100.0,
    };

    let _signals = strategy_actor_ref
        .ask(MarketDataUpdate {
            symbol: "ETH".to_string(),
            data: candle,
        })
        .await
        .expect("Actor call failed");

    // 5. Verify Order Created in DB
    // Give some time for async execution (mailbox processing)
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let orders = sqlx::query!("SELECT * FROM orders WHERE symbol = ?", "ETH")
        .fetch_all(&app.db_pool)
        .await
        .expect("Failed to fetch orders");

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].side, "buy");
    assert_eq!(orders[0].status, "filled"); // mocked execution fills immediately
}
