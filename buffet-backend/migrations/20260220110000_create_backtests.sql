CREATE TABLE IF NOT EXISTS backtests (
    id TEXT PRIMARY KEY NOT NULL,
    strategy_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    initial_balance REAL NOT NULL,
    final_balance REAL,
    total_return REAL,
    sharpe_ratio REAL,
    max_drawdown REAL,
    status TEXT NOT NULL, -- 'pending', 'running', 'completed', 'failed'
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (strategy_id) REFERENCES strategies (id)
);

CREATE TABLE IF NOT EXISTS backtest_trades (
    id TEXT PRIMARY KEY NOT NULL,
    backtest_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    quantity REAL NOT NULL,
    entry_price REAL NOT NULL,
    exit_price REAL,
    entry_time TEXT NOT NULL,
    exit_time TEXT,
    pnl REAL,
    percentage_return REAL,
    FOREIGN KEY (backtest_id) REFERENCES backtests (id) ON DELETE CASCADE
);
