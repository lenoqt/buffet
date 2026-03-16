use crate::actors::messages::{
    ActorError, ActorResult, LoadStrategies, MarketDataUpdate, OrderRequest, RegisterStrategy,
    SignalType, UnregisterStrategy,
};
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
        if fast_ma > slow_ma {
            Some(SignalType::Buy)
        } else {
            Some(SignalType::Sell)
        }
    }
}

/// Build a boxed `StrategyLogic` from a type string and JSON parameters string.
/// Returns `None` if the strategy type is unsupported or parameters are invalid.
fn build_strategy(strategy_type: &str, parameters: &str) -> Option<Box<dyn StrategyLogic>> {
    use crate::models::strategy::StrategyType;
    use std::str::FromStr;

    let params: serde_json::Value = serde_json::from_str(parameters).unwrap_or_default();

    match StrategyType::from_str(strategy_type) {
        Ok(StrategyType::Classical) => {
            let fast = params["fast_period"].as_u64().unwrap_or(10) as usize;
            let slow = params["slow_period"].as_u64().unwrap_or(20) as usize;
            Some(Box::new(MovingAverageCrossover::new(fast, slow)))
        }
        _ => {
            tracing::warn!(
                "Unsupported strategy type '{}', skipping",
                strategy_type
            );
            None
        }
    }
}

use crate::models::signal::Signal as SignalModel;
use sqlx::{Pool, Sqlite};

use crate::actors::OrderExecutionActor;
use crate::models::order::OrderSide;
use kameo::actor::ActorRef;

#[derive(Actor)]
#[actor(name = "StrategyExecutorActor")]
pub struct StrategyExecutorActor {
    active_strategies: HashMap<String, Box<dyn StrategyLogic>>,
    /// Maps strategy_id -> list of subscribed symbols (empty = all symbols)
    strategy_symbols: HashMap<String, Vec<String>>,
    pool: Pool<Sqlite>,
    execution_actor: ActorRef<OrderExecutionActor>,
}

impl StrategyExecutorActor {
    pub fn new(pool: Pool<Sqlite>, execution_actor: ActorRef<OrderExecutionActor>) -> Self {
        Self {
            active_strategies: HashMap::new(),
            strategy_symbols: HashMap::new(),
            pool,
            execution_actor,
        }
    }

    pub fn register_strategy(&mut self, id: String, strategy: Box<dyn StrategyLogic>) {
        self.active_strategies.insert(id, strategy);
    }
}

// ── MarketDataUpdate ─────────────────────────────────────────────────────────

impl Message<MarketDataUpdate> for StrategyExecutorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: MarketDataUpdate,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Process all strategies with the new data
        for (id, strategy) in &mut self.active_strategies {
            // If the strategy has symbol subscriptions, only process matching symbols
            let subscribed = self
                .strategy_symbols
                .get(id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if !subscribed.is_empty() && !subscribed.contains(&msg.symbol) {
                continue;
            }

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

// ── LoadStrategies ───────────────────────────────────────────────────────────

impl Message<LoadStrategies> for StrategyExecutorActor {
    type Reply = ActorResult<usize>;

    async fn handle(
        &mut self,
        _msg: LoadStrategies,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let strategies =
            crate::models::strategy::Strategy::find_by_status("active", &self.pool)
                .await
                .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        let mut loaded = 0usize;
        for s in strategies {
            if let Some(logic) = build_strategy(&s.strategy_type, &s.parameters) {
                self.active_strategies.insert(s.id.clone(), logic);

                // Parse the symbols JSON array stored in the DB
                let symbols: Vec<String> =
                    serde_json::from_str(&s.symbols).unwrap_or_default();
                self.strategy_symbols.insert(s.id, symbols);

                loaded += 1;
            }
        }

        tracing::info!("Loaded {} active strategies from database", loaded);
        Ok(loaded)
    }
}

// ── RegisterStrategy ─────────────────────────────────────────────────────────

impl Message<RegisterStrategy> for StrategyExecutorActor {
    type Reply = ActorResult<()>;

    async fn handle(
        &mut self,
        msg: RegisterStrategy,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match build_strategy(&msg.strategy_type, &msg.parameters) {
            Some(logic) => {
                // Parse symbols from the parameters JSON if present; default to empty (= all)
                let params: serde_json::Value =
                    serde_json::from_str(&msg.parameters).unwrap_or_default();
                let symbols: Vec<String> = params["symbols"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                self.active_strategies.insert(msg.strategy_id.clone(), logic);
                self.strategy_symbols.insert(msg.strategy_id.clone(), symbols);

                tracing::info!("Registered strategy '{}'", msg.strategy_id);
                Ok(())
            }
            None => Err(ActorError::InvalidInput(format!(
                "Unsupported strategy type '{}'",
                msg.strategy_type
            ))),
        }
    }
}

// ── UnregisterStrategy ───────────────────────────────────────────────────────

impl Message<UnregisterStrategy> for StrategyExecutorActor {
    type Reply = ActorResult<()>;

    async fn handle(
        &mut self,
        msg: UnregisterStrategy,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let removed = self.active_strategies.remove(&msg.strategy_id).is_some();
        self.strategy_symbols.remove(&msg.strategy_id);

        if removed {
            tracing::info!("Unregistered strategy '{}'", msg.strategy_id);
        } else {
            tracing::warn!(
                "Attempted to unregister unknown strategy '{}'",
                msg.strategy_id
            );
        }

        Ok(())
    }
}
