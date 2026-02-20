use crate::actors::messages::{ActorError, ActorResult, RunBacktest, SignalType, TimeSeriesRef};
use crate::actors::storage::{QueryOHLCV, TimeSeriesStorageActor};
use crate::actors::strategy::{MovingAverageCrossover, StrategyLogic};
use crate::models::backtest::{Backtest, BacktestStatus, BacktestTrade};
use crate::models::strategy::{Strategy, StrategyType};
use crate::utils::metrics::{calculate_max_drawdown, calculate_sharpe_ratio};
use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use tracing::info;

#[derive(Actor)]
#[actor(name = "BacktestActor")]
pub struct BacktestActor {
    pool: Pool<Sqlite>,
    storage_actor: ActorRef<TimeSeriesStorageActor>,
}

impl BacktestActor {
    pub fn new(pool: Pool<Sqlite>, storage_actor: ActorRef<TimeSeriesStorageActor>) -> Self {
        Self {
            pool,
            storage_actor,
        }
    }
}

impl Message<RunBacktest> for BacktestActor {
    type Reply = ActorResult<()>;

    async fn handle(
        &mut self,
        msg: RunBacktest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let backtest_id = msg.backtest_id;

        // 1. Fetch Backtest metadata
        let backtest = Backtest::find_by_id(&backtest_id, &self.pool)
            .await
            .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        // 2. Update status to Running
        let _ =
            Backtest::update_status(&backtest_id, BacktestStatus::Running, None, &self.pool).await;

        // 3. Fetch Strategy metadata
        let strategy_model = Strategy::find_by_id(&backtest.strategy_id, &self.pool)
            .await
            .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        // 4. Instantiate strategy logic
        let strategy_type = StrategyType::from_str(&strategy_model.strategy_type)
            .map_err(|e| ActorError::InvalidInput(e))?;

        let mut strategy: Box<dyn StrategyLogic> = match strategy_type {
            StrategyType::Classical => {
                let params: serde_json::Value = serde_json::from_str(&strategy_model.parameters)
                    .map_err(|e| ActorError::InvalidInput(e.to_string()))?;

                let fast = params["fast_period"].as_u64().unwrap_or(10) as usize;
                let slow = params["slow_period"].as_u64().unwrap_or(20) as usize;

                Box::new(MovingAverageCrossover::new(fast, slow))
            }
            _ => {
                let err_msg = "Unsupported strategy type for backtesting".to_string();
                let _ = Backtest::update_status(
                    &backtest_id,
                    BacktestStatus::Failed,
                    Some(err_msg.clone()),
                    &self.pool,
                )
                .await;
                return Err(ActorError::InvalidInput(err_msg));
            }
        };

        // 5. Query historical data
        let ohlcv_result = self
            .storage_actor
            .ask(QueryOHLCV {
                symbol: backtest.symbol.clone(),
                ts_ref: TimeSeriesRef::new(
                    "ohlcv".to_string(),
                    vec![],
                    backtest.start_time,
                    backtest.end_time,
                ),
            })
            .await;

        let ohlcv_data = match ohlcv_result {
            Ok(data) => data,
            Err(e) => {
                let err_msg = format!("Backtest failed - Storage error: {}", e);
                let _ = Backtest::update_status(
                    &backtest_id,
                    BacktestStatus::Failed,
                    Some(err_msg.clone()),
                    &self.pool,
                )
                .await;
                return Err(ActorError::Internal(err_msg));
            }
        };

        if ohlcv_data.is_empty() {
            let err_msg = "No data found for the given period".to_string();
            let _ = Backtest::update_status(
                &backtest_id,
                BacktestStatus::Failed,
                Some(err_msg.clone()),
                &self.pool,
            )
            .await;
            return Err(ActorError::InvalidInput(err_msg));
        }

        info!(
            "Starting backtest simulation for {} with {} data points",
            backtest_id,
            ohlcv_data.len()
        );

        // 6. Run Simulation
        let mut balance = backtest.initial_balance;
        let mut equity_curve = vec![balance];
        let mut returns = Vec::new();
        let mut current_position_qty = 0.0;
        let mut active_trade_id: Option<String> = None;

        for candle in ohlcv_data {
            let signal = strategy.update(&candle);

            if let Some(signal_type) = signal {
                match signal_type {
                    SignalType::Buy if current_position_qty == 0.0 => {
                        current_position_qty = balance / candle.close;
                        balance = 0.0;

                        let trade_result = BacktestTrade::create(
                            &backtest_id,
                            &backtest.symbol,
                            "buy",
                            current_position_qty,
                            candle.close,
                            candle.timestamp,
                            &self.pool,
                        )
                        .await;

                        match trade_result {
                            Ok(trade) => active_trade_id = Some(trade.id),
                            Err(e) => {
                                tracing::error!("Failed to create backtest trade record: {}", e)
                            }
                        }
                    }
                    SignalType::Sell if current_position_qty > 0.0 => {
                        let sell_value = current_position_qty * candle.close;
                        balance = sell_value;

                        if let Some(trade_id) = active_trade_id.take() {
                            match BacktestTrade::find_by_id(&trade_id, &self.pool).await {
                                Ok(trade) => {
                                    let pnl = (candle.close - trade.entry_price) * trade.quantity;
                                    let pct_return =
                                        (candle.close - trade.entry_price) / trade.entry_price;
                                    returns.push(pct_return);

                                    let _ = BacktestTrade::close_trade(
                                        &trade_id,
                                        candle.close,
                                        candle.timestamp,
                                        pnl,
                                        pct_return,
                                        &self.pool,
                                    )
                                    .await;
                                }
                                Err(e) => tracing::error!(
                                    "Failed to find trade {} during backtest: {}",
                                    trade_id,
                                    e
                                ),
                            }
                        }

                        current_position_qty = 0.0;
                    }
                    _ => {}
                }
            }

            let current_equity = balance + (current_position_qty * candle.close);
            equity_curve.push(current_equity);
        }

        let final_equity = equity_curve.last().copied().unwrap_or(balance);

        // 7. Calculate Metrics
        let total_return = (final_equity - backtest.initial_balance) / backtest.initial_balance;
        let sharpe = calculate_sharpe_ratio(&returns, 0.0);
        let mdd = calculate_max_drawdown(&equity_curve);

        // 8. Persist Results
        Backtest::update_results(
            &backtest_id,
            final_equity,
            total_return,
            sharpe,
            mdd,
            &self.pool,
        )
        .await
        .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        info!(
            "Backtest {} completed. Final Equity: {:.2}, Return: {:.2}%",
            backtest_id,
            final_equity,
            total_return * 100.0
        );

        Ok(())
    }
}
