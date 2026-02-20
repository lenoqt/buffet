use crate::helpers::spawn_app;
use buffet_backend::actors::strategy::StrategyLogic;
use buffet_backend::actors::{
    StrategyExecutorActor,
    messages::{MarketDataUpdate, SignalType},
};
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
async fn test_strategy_executor_persists_signals() {
    // 1. Spawn app (gets us DB pools)
    let app = spawn_app().await;

    // 2. Setup actors
    let execution_actor = buffet_backend::actors::OrderExecutionActor::spawn_with_mailbox(
        buffet_backend::actors::OrderExecutionActor::new(app.db_pool.clone()),
        mailbox::bounded(10),
    );
    let mut executor = StrategyExecutorActor::new(app.db_pool.clone(), execution_actor.clone());
    executor.register_strategy(
        "mock_persistent".to_string(),
        Box::new(MockStrategy {
            should_signal: true,
        }),
    );

    let actor_ref = StrategyExecutorActor::spawn_with_mailbox(executor, mailbox::bounded(10));

    // 3. Send MarketDataUpdate (fire-and-forget)
    let candle = buffet_backend::models::market_data::OHLCV {
        timestamp: chrono::Utc::now(),
        open: 100.0,
        high: 100.0,
        low: 100.0,
        close: 100.0,
        volume: 100.0,
    };

    actor_ref
        .tell(MarketDataUpdate {
            symbol: "BTC".to_string(),
            data: candle,
        })
        .send()
        .await
        .expect("Failed to send MarketDataUpdate");

    // 4. Wait for async processing
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 5. Verify signal persisted in DB
    let saved_signals = sqlx::query!(
        "SELECT * FROM signals WHERE strategy_id = ?",
        "mock_persistent"
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch signals");

    assert_eq!(saved_signals.len(), 1);
    assert_eq!(saved_signals[0].signal_type, "buy");
    assert_eq!(saved_signals[0].symbol, "BTC");
}
