use async_trait::async_trait;
use crate::models::market_data::OHLCV;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("No data returned for symbol {0}")]
    NoData(String),
    #[error("Rate limited")]
    RateLimited,
}

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<OHLCV>, ProviderError>;
}

pub mod normalize;
pub mod yahoo;
pub use yahoo::YahooProvider;
