use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
        }
    }
}

impl std::str::FromStr for OrderSide {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buy" => Ok(OrderSide::Buy),
            "sell" => Ok(OrderSide::Sell),
            _ => Err(format!("Invalid order side: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Open,
    Filled,
    Cancelled,
    Rejected,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Open => write!(f, "open"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(OrderStatus::Open),
            "filled" => Ok(OrderStatus::Filled),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "rejected" => Ok(OrderStatus::Rejected),
            _ => Err(format!("Invalid order status: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: String,
    pub signal_id: Option<String>,
    pub symbol: String,
    pub side: String, // Stored as string
    pub quantity: f64,
    pub price: Option<f64>,
    pub status: String, // Stored as string
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub async fn create(
        signal_id: Option<String>,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: Option<f64>,
        pool: &Pool<Sqlite>,
    ) -> Result<Order> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let side_str = side.to_string();
        let status_str = OrderStatus::Open.to_string();

        sqlx::query!(
            r#"
            INSERT INTO orders (id, signal_id, symbol, side, quantity, price, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            signal_id,
            symbol,
            side_str,
            quantity,
            price,
            status_str,
            now,
            now
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(&id, pool).await
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<Order> {
        let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Order with ID {} not found", id)))?;

        Ok(order)
    }

    pub async fn update_status(
        id: &str,
        status: OrderStatus,
        pool: &Pool<Sqlite>,
    ) -> Result<Order> {
        let now = Utc::now();
        let status_str = status.to_string();

        sqlx::query!(
            r#"
            UPDATE orders
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
            status_str,
            now,
            id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(id, pool).await
    }
}
