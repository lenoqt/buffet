-- Create strategies table
CREATE TABLE IF NOT EXISTS strategies (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    strategy_type TEXT NOT NULL,
    parameters TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_strategies_type ON strategies(strategy_type);
CREATE INDEX IF NOT EXISTS idx_strategies_created_at ON strategies(created_at);
