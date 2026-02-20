use crate::actors::messages::SignalType;
use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Signal {
    pub id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub signal_type: String,
    pub metadata: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Signal {
    pub async fn create(
        strategy_id: &str,
        symbol: &str,
        signal_type: SignalType,
        timestamp: DateTime<Utc>,
        metadata: Option<String>,
        pool: &Pool<Sqlite>,
    ) -> Result<Signal> {
        let id = Uuid::new_v4().to_string();
        let signal_type_str = signal_type.to_string();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO signals (id, strategy_id, symbol, signal_type, timestamp, metadata, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            strategy_id,
            symbol,
            signal_type_str,
            timestamp,
            metadata,
            now
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(&id, pool).await
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<Signal> {
        let signal = sqlx::query_as::<_, Signal>("SELECT * FROM signals WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Signal with ID {} not found", id)))?;

        Ok(signal)
    }

    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<Signal>> {
        let signals = sqlx::query_as::<_, Signal>("SELECT * FROM signals ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(signals)
    }
}
