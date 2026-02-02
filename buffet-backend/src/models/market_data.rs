use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OHLCV (Open, High, Low, Close, Volume) candlestick data
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OHLCV {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl OHLCV {
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

/// Ticker information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Ticker {
    pub symbol: String,
    pub exchange: Option<String>,
    pub asset_type: AssetType,
}

impl Ticker {
    pub fn new(symbol: String, exchange: Option<String>, asset_type: AssetType) -> Self {
        Self {
            symbol,
            exchange,
            asset_type,
        }
    }

    pub fn stock(symbol: String) -> Self {
        Self {
            symbol,
            exchange: None,
            asset_type: AssetType::Stock,
        }
    }

    pub fn crypto(symbol: String) -> Self {
        Self {
            symbol,
            exchange: None,
            asset_type: AssetType::Crypto,
        }
    }
}

/// Asset type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetType {
    Stock,
    Crypto,
    Forex,
    Commodity,
    Index,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Stock => write!(f, "stock"),
            AssetType::Crypto => write!(f, "crypto"),
            AssetType::Forex => write!(f, "forex"),
            AssetType::Commodity => write!(f, "commodity"),
            AssetType::Index => write!(f, "index"),
        }
    }
}
