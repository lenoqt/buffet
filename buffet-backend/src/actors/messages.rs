// Common message types for actor communication
use serde::{Deserialize, Serialize};

/// Common result type for actor messages
pub type ActorResult<T> = Result<T, ActorError>;

/// Common error type for actor operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorError {
    /// Database operation failed
    DatabaseError(String),
    /// TSDB operation failed
    TsdbError(String),
    /// Invalid input or configuration
    InvalidInput(String),
    /// Actor not found or unavailable
    ActorUnavailable(String),
    /// Operation timed out
    Timeout,
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ActorError::TsdbError(msg) => write!(f, "TSDB error: {}", msg),
            ActorError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ActorError::ActorUnavailable(msg) => write!(f, "Actor unavailable: {}", msg),
            ActorError::Timeout => write!(f, "Operation timed out"),
            ActorError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ActorError {}

/// Reference to a time-series in the TSDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesRef {
    /// Measurement name in TSDB
    pub measurement: String,
    /// Tags for filtering
    pub tags: Vec<(String, String)>,
    /// Start time of the series
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time of the series
    pub end_time: chrono::DateTime<chrono::Utc>,
}

impl TimeSeriesRef {
    pub fn new(
        measurement: String,
        tags: Vec<(String, String)>,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            measurement,
            tags,
            start_time,
            end_time,
        }
    }
}
