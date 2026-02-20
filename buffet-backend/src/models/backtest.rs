use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BacktestStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for BacktestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BacktestStatus::Pending => write!(f, "pending"),
            BacktestStatus::Running => write!(f, "running"),
            BacktestStatus::Completed => write!(f, "completed"),
            BacktestStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Backtest {
    pub id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: f64,
    pub final_balance: Option<f64>,
    pub total_return: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BacktestTrade {
    pub id: String,
    pub backtest_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub pnl: Option<f64>,
    pub percentage_return: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBacktestDto {
    pub strategy_id: String,
    pub symbol: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: f64,
}

impl Backtest {
    pub async fn create(dto: CreateBacktestDto, pool: &Pool<Sqlite>) -> Result<Backtest> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let status = BacktestStatus::Pending.to_string();

        sqlx::query!(
            r#"
            INSERT INTO backtests (id, strategy_id, symbol, start_time, end_time, initial_balance, status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            dto.strategy_id,
            dto.symbol,
            dto.start_time,
            dto.end_time,
            dto.initial_balance,
            status,
            now
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(&id, pool).await
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<Backtest> {
        let backtest = sqlx::query_as::<_, Backtest>("SELECT * FROM backtests WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Backtest with ID {} not found", id)))?;

        Ok(backtest)
    }

    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<Backtest>> {
        let backtests =
            sqlx::query_as::<_, Backtest>("SELECT * FROM backtests ORDER BY created_at DESC")
                .fetch_all(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(backtests)
    }

    pub async fn update_status(
        id: &str,
        status: BacktestStatus,
        error_message: Option<String>,
        pool: &Pool<Sqlite>,
    ) -> Result<()> {
        let status_str = status.to_string();
        sqlx::query!(
            "UPDATE backtests SET status = ?, error_message = ? WHERE id = ?",
            status_str,
            error_message,
            id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn update_results(
        id: &str,
        final_balance: f64,
        total_return: f64,
        sharpe_ratio: f64,
        max_drawdown: f64,
        pool: &Pool<Sqlite>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE backtests 
            SET final_balance = ?, total_return = ?, sharpe_ratio = ?, max_drawdown = ?, status = 'completed'
            WHERE id = ?
            "#,
            final_balance,
            total_return,
            sharpe_ratio,
            max_drawdown,
            id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}

impl BacktestTrade {
    pub async fn create(
        backtest_id: &str,
        symbol: &str,
        side: &str,
        quantity: f64,
        entry_price: f64,
        entry_time: DateTime<Utc>,
        pool: &Pool<Sqlite>,
    ) -> Result<BacktestTrade> {
        let id = Uuid::new_v4().to_string();

        sqlx::query!(
            r#"
            INSERT INTO backtest_trades (id, backtest_id, symbol, side, quantity, entry_price, entry_time)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            backtest_id,
            symbol,
            side,
            quantity,
            entry_price,
            entry_time
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(&id, pool).await
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<BacktestTrade> {
        let trade =
            sqlx::query_as::<_, BacktestTrade>("SELECT * FROM backtest_trades WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::Database)?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Backtest trade with ID {} not found", id))
                })?;

        Ok(trade)
    }

    pub async fn find_by_backtest(
        backtest_id: &str,
        pool: &Pool<Sqlite>,
    ) -> Result<Vec<BacktestTrade>> {
        let trades = sqlx::query_as::<_, BacktestTrade>(
            "SELECT * FROM backtest_trades WHERE backtest_id = ? ORDER BY entry_time ASC",
        )
        .bind(backtest_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(trades)
    }

    pub async fn close_trade(
        id: &str,
        exit_price: f64,
        exit_time: DateTime<Utc>,
        pnl: f64,
        percentage_return: f64,
        pool: &Pool<Sqlite>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE backtest_trades 
            SET exit_price = ?, exit_time = ?, pnl = ?, percentage_return = ?
            WHERE id = ?
            "#,
            exit_price,
            exit_time,
            pnl,
            percentage_return,
            id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}
