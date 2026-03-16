ALTER TABLE backtests ADD COLUMN commission_rate REAL NOT NULL DEFAULT 0.001;
ALTER TABLE backtests ADD COLUMN slippage_bps REAL NOT NULL DEFAULT 10.0;
ALTER TABLE backtests ADD COLUMN run_config TEXT;
ALTER TABLE backtests ADD COLUMN trade_count INTEGER;
ALTER TABLE backtests ADD COLUMN win_rate REAL;
ALTER TABLE backtests ADD COLUMN profit_factor REAL;
