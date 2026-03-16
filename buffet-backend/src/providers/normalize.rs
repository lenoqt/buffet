use crate::models::market_data::OHLCV;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Negative price at timestamp {timestamp}")]
    NegativePrice { timestamp: String },
    #[error("OHLCV integrity violation at {timestamp}: high={high} < max(open={open}, close={close})")]
    OhlcvIntegrity {
        timestamp: String,
        high: f64,
        open: f64,
        close: f64,
    },
    #[error("Negative volume at {timestamp}")]
    NegativeVolume { timestamp: String },
}

/// Sort by timestamp, deduplicate, and validate every OHLCV record.
pub fn normalize_ohlcv(mut data: Vec<OHLCV>) -> Result<Vec<OHLCV>, ValidationError> {
    // Sort ascending by timestamp
    data.sort_by_key(|o| o.timestamp);

    // Remove consecutive records with the same timestamp (keep the first occurrence)
    data.dedup_by_key(|o| o.timestamp);

    // Validate each record
    for ohlcv in &data {
        let ts = ohlcv.timestamp.to_rfc3339();

        if ohlcv.open < 0.0 || ohlcv.high < 0.0 || ohlcv.low < 0.0 || ohlcv.close < 0.0 {
            return Err(ValidationError::NegativePrice { timestamp: ts });
        }

        if ohlcv.high < ohlcv.open.max(ohlcv.close) {
            return Err(ValidationError::OhlcvIntegrity {
                timestamp: ts,
                high: ohlcv.high,
                open: ohlcv.open,
                close: ohlcv.close,
            });
        }

        if ohlcv.volume < 0.0 {
            return Err(ValidationError::NegativeVolume { timestamp: ts });
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn make_ohlcv(ts_secs: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> OHLCV {
        OHLCV {
            timestamp: Utc.timestamp_opt(ts_secs, 0).single().expect("valid timestamp"),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn test_valid_single_record() {
        let data = vec![make_ohlcv(1_000_000, 100.0, 110.0, 90.0, 105.0, 5000.0)];
        let result = normalize_ohlcv(data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_sorts_by_timestamp() {
        let data = vec![
            make_ohlcv(3_000_000, 100.0, 110.0, 90.0, 105.0, 1000.0),
            make_ohlcv(1_000_000, 100.0, 110.0, 90.0, 105.0, 1000.0),
            make_ohlcv(2_000_000, 100.0, 110.0, 90.0, 105.0, 1000.0),
        ];
        let result = normalize_ohlcv(data).unwrap();
        assert_eq!(result[0].timestamp.timestamp(), 1_000_000);
        assert_eq!(result[1].timestamp.timestamp(), 2_000_000);
        assert_eq!(result[2].timestamp.timestamp(), 3_000_000);
    }

    #[test]
    fn test_deduplicates_timestamps() {
        let data = vec![
            make_ohlcv(1_000_000, 100.0, 110.0, 90.0, 105.0, 1000.0),
            make_ohlcv(1_000_000, 200.0, 220.0, 180.0, 210.0, 2000.0),
            make_ohlcv(2_000_000, 100.0, 110.0, 90.0, 105.0, 1000.0),
        ];
        let result = normalize_ohlcv(data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].timestamp.timestamp(), 1_000_000);
        assert_eq!(result[1].timestamp.timestamp(), 2_000_000);
    }

    #[test]
    fn test_negative_open_fails() {
        let data = vec![make_ohlcv(1_000_000, -1.0, 110.0, 90.0, 105.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::NegativePrice { .. })));
    }

    #[test]
    fn test_negative_high_fails() {
        let data = vec![make_ohlcv(1_000_000, 100.0, -5.0, 90.0, 105.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::NegativePrice { .. })));
    }

    #[test]
    fn test_negative_low_fails() {
        let data = vec![make_ohlcv(1_000_000, 100.0, 110.0, -1.0, 105.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::NegativePrice { .. })));
    }

    #[test]
    fn test_negative_close_fails() {
        let data = vec![make_ohlcv(1_000_000, 100.0, 110.0, 90.0, -5.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::NegativePrice { .. })));
    }

    #[test]
    fn test_high_less_than_open_fails() {
        // high=95 < open=100 → integrity violation
        let data = vec![make_ohlcv(1_000_000, 100.0, 95.0, 90.0, 93.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::OhlcvIntegrity { .. })));
    }

    #[test]
    fn test_high_less_than_close_fails() {
        // high=98 < close=105 → integrity violation
        let data = vec![make_ohlcv(1_000_000, 90.0, 98.0, 85.0, 105.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::OhlcvIntegrity { .. })));
    }

    #[test]
    fn test_negative_volume_fails() {
        let data = vec![make_ohlcv(1_000_000, 100.0, 110.0, 90.0, 105.0, -1.0)];
        let result = normalize_ohlcv(data);
        assert!(matches!(result, Err(ValidationError::NegativeVolume { .. })));
    }

    #[test]
    fn test_zero_values_are_valid() {
        // Zero prices and volume should pass (e.g. some exotic data points)
        let data = vec![make_ohlcv(1_000_000, 0.0, 0.0, 0.0, 0.0, 0.0)];
        let result = normalize_ohlcv(data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_input_returns_empty() {
        let result = normalize_ohlcv(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_multiple_valid_records() {
        let data = vec![
            make_ohlcv(1_000_000, 100.0, 115.0, 98.0, 112.0, 3000.0),
            make_ohlcv(2_000_000, 112.0, 120.0, 110.0, 118.0, 4500.0),
            make_ohlcv(3_000_000, 118.0, 125.0, 115.0, 122.0, 2800.0),
        ];
        let result = normalize_ohlcv(data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_high_equal_to_max_open_close_is_valid() {
        // high == open == close → boundary case, should pass
        let data = vec![make_ohlcv(1_000_000, 105.0, 105.0, 95.0, 105.0, 1000.0)];
        let result = normalize_ohlcv(data);
        assert!(result.is_ok());
    }
}
