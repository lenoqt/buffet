CREATE TABLE IF NOT EXISTS positions (
    id TEXT PRIMARY KEY NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    quantity REAL NOT NULL DEFAULT 0.0,
    avg_entry_price REAL NOT NULL DEFAULT 0.0,
    unrealized_pnl REAL NOT NULL DEFAULT 0.0,
    realized_pnl REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL DEFAULT 'open',
    opened_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
