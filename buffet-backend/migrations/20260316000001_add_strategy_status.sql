ALTER TABLE strategies ADD COLUMN status TEXT NOT NULL DEFAULT 'inactive';
CREATE INDEX IF NOT EXISTS idx_strategies_status ON strategies(status);
