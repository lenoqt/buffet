CREATE TABLE orders (
    id TEXT PRIMARY KEY NOT NULL,
    signal_id TEXT,  -- Can be null if manual order
    symbol TEXT NOT NULL,
    side TEXT NOT NULL, -- 'buy' or 'sell'
    quantity REAL NOT NULL,
    price REAL, -- Null for market orders
    status TEXT NOT NULL, -- 'open', 'filled', 'cancelled', 'rejected'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
