use async_trait::async_trait;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;

use crate::models::market_data::OHLCV;
use crate::providers::{MarketDataProvider, ProviderError};

pub struct YahooProvider;

impl YahooProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for YahooProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a `chrono::DateTime<Utc>` to `time::OffsetDateTime`.
fn chrono_to_offset(dt: DateTime<Utc>) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(dt.timestamp())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

#[async_trait]
impl MarketDataProvider for YahooProvider {
    fn name(&self) -> &str {
        "Yahoo Finance"
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<OHLCV>, ProviderError> {
        let connector = yahoo_finance_api::YahooConnector::new()
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let start_odt = chrono_to_offset(start);
        let end_odt = chrono_to_offset(end);

        let response = connector
            .get_quote_history(symbol, start_odt, end_odt)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.to_lowercase().contains("too many") || msg.contains("429") {
                    ProviderError::RateLimited
                } else {
                    ProviderError::Http(msg)
                }
            })?;

        let quotes = response
            .quotes()
            .map_err(|e| ProviderError::Parse(e.to_string()))?;

        if quotes.is_empty() {
            return Err(ProviderError::NoData(symbol.to_string()));
        }

        // Without the `decimal` feature, yahoo_finance_api's `Decimal` type alias
        // is simply `f64`, so open/high/low/close are plain f64 values.
        let ohlcv: Vec<OHLCV> = quotes
            .into_iter()
            .map(|q| OHLCV {
                timestamp: DateTime::from_timestamp(q.timestamp as i64, 0)
                    .unwrap_or_default(),
                open: q.open,
                high: q.high,
                low: q.low,
                close: q.close,
                volume: q.volume as f64,
            })
            .collect();

        Ok(ohlcv)
    }
}
