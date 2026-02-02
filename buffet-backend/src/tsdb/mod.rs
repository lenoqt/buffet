use crate::error::{AppError, Result};
use crate::models::market_data::OHLCV;
use sqlx::{Pool, Postgres, types::chrono};

pub struct TimescaleDb {
    pool: Pool<Postgres>,
}

impl TimescaleDb {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Initialize TimescaleDB (create extension and hypertables if they don't exist)
    pub async fn setup(&self) -> Result<()> {
        // Create extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE")
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        // Create OHLCV table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ohlcv (
                time TIMESTAMPTZ NOT NULL,
                symbol TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                open DOUBLE PRECISION NOT NULL,
                high DOUBLE PRECISION NOT NULL,
                low DOUBLE PRECISION NOT NULL,
                close DOUBLE PRECISION NOT NULL,
                volume DOUBLE PRECISION NOT NULL
            )
        "#,
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        // Convert to hypertable (error if already a hypertable, so we check first)
        let is_hypertable: (bool,) = sqlx::query_as(
            "SELECT count(*) > 0 FROM timescaledb_information.hypertables WHERE hypertable_name = 'ohlcv'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        if !is_hypertable.0 {
            sqlx::query("SELECT create_hypertable('ohlcv', 'time')")
                .execute(&self.pool)
                .await
                .map_err(AppError::Database)?;
        }

        Ok(())
    }

    /// Insert market data points
    pub async fn insert_ohlcv(&self, symbol: &str, asset_type: &str, data: &[OHLCV]) -> Result<()> {
        for point in data {
            sqlx::query(
                r#"
                INSERT INTO ohlcv (time, symbol, asset_type, open, high, low, close, volume)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(point.timestamp)
            .bind(symbol)
            .bind(asset_type)
            .bind(point.open)
            .bind(point.high)
            .bind(point.low)
            .bind(point.close)
            .bind(point.volume)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }
        Ok(())
    }

    /// Query market data
    pub async fn query_ohlcv(
        &self,
        symbol: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<OHLCV>> {
        let rows = sqlx::query_as::<_, OHLCV>(
            r#"
            SELECT time as timestamp, open, high, low, close, volume
            FROM ohlcv
            WHERE symbol = $1 AND time >= $2 AND time <= $3
            ORDER BY time ASC
            "#,
        )
        .bind(symbol)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rows)
    }
}
