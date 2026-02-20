use crate::actors::messages::{MarketDataUpdate, SignalType};
use crate::models::market_data::OHLCV;
use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;

// Trait for strategy logic
pub trait StrategyLogic: Send + Sync {
    fn update(&mut self, data: &OHLCV) -> Option<SignalType>;
}

// Simple Moving Average Crossover Strategy
pub struct MovingAverageCrossover {
    fast_period: usize,
    slow_period: usize,
    prices: Vec<f64>,
}

impl MovingAverageCrossover {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            prices: Vec::new(),
        }
    }

    fn calculate_sma(&self, period: usize) -> Option<f64> {
        if self.prices.len() < period {
            return None;
        }
        let sum: f64 = self.prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }
}

impl StrategyLogic for MovingAverageCrossover {
    fn update(&mut self, data: &OHLCV) -> Option<SignalType> {
        self.prices.push(data.close);

        // Keep only enough history for the slow period
        if self.prices.len() > self.slow_period + 1 {
            self.prices.remove(0);
        }

        let fast_ma = self.calculate_sma(self.fast_period)?;
        let slow_ma = self.calculate_sma(self.slow_period)?;

        // Simple logic: if fast crosses above slow -> Buy
        // if fast crosses below slow -> Sell
        // This requires previous values to detect crossover, omitted for brevity in first pass
        // Just returning based on current relation (which is state-based, not crossover-based)
        if fast_ma > slow_ma {
            Some(SignalType::Buy)
        } else {
            Some(SignalType::Sell)
        }
    }
}

use crate::models::signal::Signal as SignalModel;
use sqlx::{Pool, Sqlite};

use crate::actors::OrderExecutionActor;
use crate::actors::messages::OrderRequest;
use crate::models::order::OrderSide;
use kameo::actor::ActorRef;

#[derive(Actor)]
#[actor(name = "StrategyExecutorActor")]
pub struct StrategyExecutorActor {
    active_strategies: HashMap<String, Box<dyn StrategyLogic>>,
    pool: Pool<Sqlite>,
    execution_actor: ActorRef<OrderExecutionActor>,
}

impl StrategyExecutorActor {
    pub fn new(pool: Pool<Sqlite>, execution_actor: ActorRef<OrderExecutionActor>) -> Self {
        Self {
            active_strategies: HashMap::new(),
            pool,
            execution_actor,
        }
    }

    pub fn register_strategy(&mut self, id: String, strategy: Box<dyn StrategyLogic>) {
        self.active_strategies.insert(id, strategy);
    }
}

impl Message<MarketDataUpdate> for StrategyExecutorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: MarketDataUpdate,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Process all strategies with the new data
        for (id, strategy) in &mut self.active_strategies {
            if let Some(signal_type) = strategy.update(&msg.data) {
                let timestamp = chrono::Utc::now();

                // Persist signal to DB
                let created_signal = match SignalModel::create(
                    id,
                    &msg.symbol,
                    signal_type,
                    timestamp,
                    None,
                    &self.pool,
                )
                .await
                {
                    Ok(s) => Some(s),
                    Err(e) => {
                        tracing::error!("Failed to persist signal: {:?}", e);
                        None
                    }
                };

                // Forward order request if signal persisted and is actionable
                if let Some(signal_record) = created_signal {
                    let side = match signal_type {
                        SignalType::Buy => Some(OrderSide::Buy),
                        SignalType::Sell => Some(OrderSide::Sell),
                        SignalType::Hold => None,
                    };

                    if let Some(order_side) = side {
                        let _ = self
                            .execution_actor
                            .tell(OrderRequest {
                                signal_id: signal_record.id,
                                symbol: msg.symbol.clone(),
                                side: order_side,
                                quantity: 1.0,
                                price: None,
                            })
                            .send()
                            .await;
                    }
                }
            }
        }
    }
}
