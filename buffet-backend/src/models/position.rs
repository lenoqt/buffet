use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionStatus {
    Open,
    Closed,
}

impl std::fmt::Display for PositionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionStatus::Open => write!(f, "open"),
            PositionStatus::Closed => write!(f, "closed"),
        }
    }
}

impl std::str::FromStr for PositionStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(PositionStatus::Open),
            "closed" => Ok(PositionStatus::Closed),
            _ => Err(format!("Invalid position status: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Position {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub status: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    /// Open a new position or update an existing one for the given symbol/side
    pub async fn open_or_update(
        symbol: &str,
        side: &str,
        fill_quantity: f64,
        fill_price: f64,
        pool: &Pool<Sqlite>,
    ) -> Result<Position> {
        // Check if there's an existing open position for this symbol/side
        let existing = sqlx::query_as::<_, Position>(
            "SELECT * FROM positions WHERE symbol = ? AND side = ? AND status = 'open'",
        )
        .bind(symbol)
        .bind(side)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        match existing {
            Some(pos) => {
                // Update existing position: weighted average entry price
                let total_qty = pos.quantity + fill_quantity;
                let new_avg =
                    (pos.avg_entry_price * pos.quantity + fill_price * fill_quantity) / total_qty;
                let now = Utc::now();

                sqlx::query(
                    "UPDATE positions SET quantity = ?, avg_entry_price = ?, updated_at = ? WHERE id = ?",
                )
                .bind(total_qty)
                .bind(new_avg)
                .bind(now)
                .bind(&pos.id)
                .execute(pool)
                .await
                .map_err(AppError::Database)?;

                Self::find_by_id(&pos.id, pool).await
            }
            None => {
                // Create new position
                let id = Uuid::new_v4().to_string();
                let now = Utc::now();
                let status = PositionStatus::Open.to_string();

                sqlx::query(
                    r#"
                    INSERT INTO positions (id, symbol, side, quantity, avg_entry_price, unrealized_pnl, realized_pnl, status, opened_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, 0.0, 0.0, ?, ?, ?)
                    "#,
                )
                .bind(&id)
                .bind(symbol)
                .bind(side)
                .bind(fill_quantity)
                .bind(fill_price)
                .bind(&status)
                .bind(now)
                .bind(now)
                .execute(pool)
                .await
                .map_err(AppError::Database)?;

                Self::find_by_id(&id, pool).await
            }
        }
    }

    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<Position>> {
        let positions =
            sqlx::query_as::<_, Position>("SELECT * FROM positions ORDER BY opened_at DESC")
                .fetch_all(pool)
                .await
                .map_err(AppError::Database)?;
        Ok(positions)
    }

    pub async fn find_open(pool: &Pool<Sqlite>) -> Result<Vec<Position>> {
        let positions = sqlx::query_as::<_, Position>(
            "SELECT * FROM positions WHERE status = 'open' ORDER BY opened_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        Ok(positions)
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<Position> {
        let position = sqlx::query_as::<_, Position>("SELECT * FROM positions WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Position with ID {} not found", id)))?;
        Ok(position)
    }

    pub async fn close(id: &str, realized_pnl: f64, pool: &Pool<Sqlite>) -> Result<Position> {
        let now = Utc::now();
        let status = PositionStatus::Closed.to_string();

        sqlx::query(
            "UPDATE positions SET status = ?, realized_pnl = ?, closed_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&status)
        .bind(realized_pnl)
        .bind(now)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(id, pool).await
    }
}
