CREATE INDEX IF NOT EXISTS idx_signals_strategy_id ON signals(strategy_id);
CREATE INDEX IF NOT EXISTS idx_signals_symbol_timestamp ON signals(symbol, timestamp);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_symbol ON orders(symbol);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at);
CREATE INDEX IF NOT EXISTS idx_positions_symbol_side_status ON positions(symbol, side, status);
CREATE INDEX IF NOT EXISTS idx_backtests_strategy_id ON backtests(strategy_id);
CREATE INDEX IF NOT EXISTS idx_backtests_status ON backtests(status);
CREATE INDEX IF NOT EXISTS idx_backtest_trades_backtest_id ON backtest_trades(backtest_id);
