use crate::actors::messages::{ActorError, ActorResult, RunBacktest, SignalType, TimeSeriesRef};
use crate::actors::storage::{QueryOHLCV, TimeSeriesStorageActor};
use crate::actors::strategy::{MovingAverageCrossover, StrategyLogic};
use crate::broker::{Broker, BacktestBroker};
use crate::models::backtest::{Backtest, BacktestStatus, BacktestTrade};
use crate::models::order::OrderSide;
use crate::models::strategy::{Strategy, StrategyType};
use crate::utils::metrics::{
    calculate_max_drawdown, calculate_profit_factor, calculate_sharpe_ratio, calculate_win_rate,
};
use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use tracing::{info, warn};

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

        // ── 1. Fetch Backtest metadata ────────────────────────────────────────
        let backtest = Backtest::find_by_id(&backtest_id, &self.pool)
            .await
            .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        // ── 2. Update status → Running ────────────────────────────────────────
        let _ =
            Backtest::update_status(&backtest_id, BacktestStatus::Running, None, &self.pool).await;

        // ── 3. Fetch Strategy metadata ────────────────────────────────────────
        let strategy_model = Strategy::find_by_id(&backtest.strategy_id, &self.pool)
            .await
            .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        // ── 4. Instantiate strategy logic ─────────────────────────────────────
        let strategy_type = StrategyType::from_str(&strategy_model.strategy_type)
            .map_err(|e| ActorError::InvalidInput(e))?;

        let strategy_params: serde_json::Value =
            serde_json::from_str(&strategy_model.parameters)
                .unwrap_or_else(|_| serde_json::json!({}));

        let (mut strategy, lookback): (Box<dyn StrategyLogic>, usize) = match strategy_type {
            StrategyType::Classical => {
                let fast = strategy_params["fast_period"].as_u64().unwrap_or(10) as usize;
                let slow = strategy_params["slow_period"].as_u64().unwrap_or(20) as usize;
                let lookback = slow;
                (Box::new(MovingAverageCrossover::new(fast, slow)), lookback)
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

        // ── 5. Build run_config snapshot ──────────────────────────────────────
        let run_config = serde_json::json!({
            "strategy_name": strategy_model.name,
            "strategy_type": strategy_model.strategy_type,
            "parameters": strategy_params,
            "commission_rate": backtest.commission_rate,
            "slippage_bps": backtest.slippage_bps,
            "symbol": backtest.symbol,
            "start_time": backtest.start_time,
            "end_time": backtest.end_time,
        })
        .to_string();

        // ── 6. Query historical data ──────────────────────────────────────────
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

        // ── 7. Edge case: insufficient data for strategy lookback ─────────────
        if ohlcv_data.len() < lookback {
            warn!(
                "Backtest {}: only {} data points but strategy requires {}. \
                 Completing with zero metrics.",
                backtest_id,
                ohlcv_data.len(),
                lookback
            );
            Backtest::update_results(
                &backtest_id,
                backtest.initial_balance,
                0.0,
                0.0,
                0.0,
                Some(run_config),
                0,
                0.0,
                0.0,
                &self.pool,
            )
            .await
            .map_err(|e| ActorError::DatabaseError(e.to_string()))?;
            return Ok(());
        }

        info!(
            "Starting backtest simulation for {} with {} data points",
            backtest_id,
            ohlcv_data.len()
        );

        // ── 8. Initialise broker and simulation state ─────────────────────────
        let mut broker =
            BacktestBroker::new(backtest.commission_rate, backtest.slippage_bps);

        let mut balance = backtest.initial_balance;
        let mut equity_curve = vec![balance];
        let mut returns: Vec<f64> = Vec::new();
        let mut trades_pnl: Vec<f64> = Vec::new();

        let mut current_position_qty = 0.0f64;
        let mut active_trade_id: Option<String> = None;

        // ── 9. Main simulation loop ───────────────────────────────────────────
        for candle in &ohlcv_data {
            let signal = strategy.update(candle);

            if let Some(signal_type) = signal {
                match signal_type {
                    SignalType::Buy if current_position_qty == 0.0 => {
                        // Set current price on broker then request a fill
                        broker.set_price(candle.close);
                        let fill = broker
                            .submit_market_order(&backtest.symbol, &OrderSide::Buy, 1.0)
                            .await
                            .map_err(|e| ActorError::Internal(e.to_string()))?;

                        // Determine how many units we can afford at the filled price
                        let commission = fill.commission.unwrap_or(0.0);
                        let affordable_qty =
                            (balance - commission) / fill.fill_price;

                        if affordable_qty <= 0.0 {
                            // Cannot afford even one unit — skip
                            continue;
                        }

                        let actual_cost = affordable_qty * fill.fill_price;
                        let actual_commission = actual_cost * backtest.commission_rate;
                        balance -= actual_cost + actual_commission;
                        current_position_qty = affordable_qty;

                        // Record trade entry
                        let trade_result = BacktestTrade::create(
                            &backtest_id,
                            &backtest.symbol,
                            "buy",
                            current_position_qty,
                            fill.fill_price,
                            candle.timestamp,
                            &self.pool,
                        )
                        .await;

                        match trade_result {
                            Ok(trade) => active_trade_id = Some(trade.id),
                            Err(e) => {
                                tracing::error!(
                                    "Failed to create backtest trade record: {}",
                                    e
                                )
                            }
                        }
                    }

                    SignalType::Sell if current_position_qty > 0.0 => {
                        // Fill the sell order through the broker
                        broker.set_price(candle.close);
                        let fill = broker
                            .submit_market_order(
                                &backtest.symbol,
                                &OrderSide::Sell,
                                current_position_qty,
                            )
                            .await
                            .map_err(|e| ActorError::Internal(e.to_string()))?;

                        let gross_proceeds = fill.fill_price * fill.fill_quantity;
                        let commission = fill.commission.unwrap_or(0.0);
                        let net_proceeds = gross_proceeds - commission;
                        balance += net_proceeds;

                        // Close out the trade record and record metrics
                        if let Some(trade_id) = active_trade_id.take() {
                            match BacktestTrade::find_by_id(&trade_id, &self.pool).await {
                                Ok(trade) => {
                                    let pnl = (fill.fill_price - trade.entry_price)
                                        * trade.quantity;
                                    let pct_return = (fill.fill_price - trade.entry_price)
                                        / trade.entry_price;
                                    returns.push(pct_return);
                                    trades_pnl.push(pnl);

                                    let _ = BacktestTrade::close_trade(
                                        &trade_id,
                                        fill.fill_price,
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

        // ── 10. Close any open position at the last candle price ──────────────
        if current_position_qty > 0.0 {
            let last_candle = ohlcv_data.last().unwrap();
            broker.set_price(last_candle.close);
            let fill = broker
                .submit_market_order(
                    &backtest.symbol,
                    &OrderSide::Sell,
                    current_position_qty,
                )
                .await
                .map_err(|e| ActorError::Internal(e.to_string()))?;

            let gross_proceeds = fill.fill_price * fill.fill_quantity;
            let commission = fill.commission.unwrap_or(0.0);
            balance += gross_proceeds - commission;

            warn!(
                "Backtest {}: simulation ended with open position; closed {} units @ {:.4}",
                backtest_id, current_position_qty, fill.fill_price
            );

            if let Some(trade_id) = active_trade_id.take() {
                if let Ok(trade) = BacktestTrade::find_by_id(&trade_id, &self.pool).await {
                    let pnl = (fill.fill_price - trade.entry_price) * trade.quantity;
                    let pct_return = (fill.fill_price - trade.entry_price) / trade.entry_price;
                    returns.push(pct_return);
                    trades_pnl.push(pnl);

                    let _ = BacktestTrade::close_trade(
                        &trade_id,
                        fill.fill_price,
                        last_candle.timestamp,
                        pnl,
                        pct_return,
                        &self.pool,
                    )
                    .await;
                }
            }

            // Update final equity curve entry
            if let Some(last) = equity_curve.last_mut() {
                *last = balance;
            }
        }

        let final_equity = balance;

        // ── 11. Calculate Metrics ─────────────────────────────────────────────
        let trade_count = trades_pnl.len() as i64;

        let (total_return, sharpe, mdd, win_rate, profit_factor) = if trade_count == 0 {
            // Zero-trade edge case
            warn!(
                "Backtest {}: simulation produced zero completed trades.",
                backtest_id
            );
            (0.0, 0.0, 0.0, 0.0, 0.0)
        } else {
            let tr = (final_equity - backtest.initial_balance) / backtest.initial_balance;
            let s = calculate_sharpe_ratio(&returns, 0.0);
            let d = calculate_max_drawdown(&equity_curve);
            let wr = calculate_win_rate(&trades_pnl);
            let pf = {
                let raw = calculate_profit_factor(&trades_pnl);
                // Persist f64::INFINITY as a very large finite number (SQLite REAL limitation)
                if raw.is_infinite() { f64::MAX } else { raw }
            };
            (tr, s, d, wr, pf)
        };

        // ── 12. Persist Results ───────────────────────────────────────────────
        Backtest::update_results(
            &backtest_id,
            final_equity,
            total_return,
            sharpe,
            mdd,
            Some(run_config),
            trade_count,
            win_rate,
            profit_factor,
            &self.pool,
        )
        .await
        .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        info!(
            "Backtest {} completed. Trades: {}, Final Equity: {:.2}, Return: {:.2}%, \
             Sharpe: {:.3}, MDD: {:.2}%, Win Rate: {:.1}%, Profit Factor: {:.3}",
            backtest_id,
            trade_count,
            final_equity,
            total_return * 100.0,
            sharpe,
            mdd * 100.0,
            win_rate * 100.0,
            profit_factor,
        );

        Ok(())
    }
}
