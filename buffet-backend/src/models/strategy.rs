use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

use crate::error::{AppError, Result};

/// Strategy metadata stored in SQLite
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub strategy_type: String, // Stored as string in DB
    pub parameters: String,    // JSON string of parameters
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Strategy type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    Classical,
    Statistical,
    MLBased,
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::Classical => write!(f, "classical"),
            StrategyType::Statistical => write!(f, "statistical"),
            StrategyType::MLBased => write!(f, "ml_based"),
        }
    }
}

impl std::str::FromStr for StrategyType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "classical" => Ok(StrategyType::Classical),
            "statistical" => Ok(StrategyType::Statistical),
            "ml_based" => Ok(StrategyType::MLBased),
            _ => Err(format!("Unknown strategy type: {}", s)),
        }
    }
}

/// DTO for creating a new strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStrategyDto {
    pub name: String,
    pub strategy_type: StrategyType,
    pub parameters: serde_json::Value,
}

/// DTO for updating a strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStrategyDto {
    pub name: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

impl Strategy {
    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<Strategy>> {
        let strategies =
            sqlx::query_as::<_, Strategy>("SELECT * FROM strategies ORDER BY created_at DESC")
                .fetch_all(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(strategies)
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<Strategy> {
        let strategy = sqlx::query_as::<_, Strategy>("SELECT * FROM strategies WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Strategy with ID {} not found", id)))?;

        Ok(strategy)
    }

    pub async fn create(dto: CreateStrategyDto, pool: &Pool<Sqlite>) -> Result<Strategy> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let strategy_type = dto.strategy_type.to_string();
        let parameters = serde_json::to_string(&dto.parameters)
            .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO strategies (id, name, strategy_type, parameters, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id,
            dto.name,
            strategy_type,
            parameters,
            now,
            now
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(&id, pool).await
    }

    pub async fn update(id: &str, dto: UpdateStrategyDto, pool: &Pool<Sqlite>) -> Result<Strategy> {
        // Check if strategy exists
        Self::find_by_id(id, pool).await?;

        let now = Utc::now();
        let mut query = String::from("UPDATE strategies SET updated_at = ? ");

        if dto.name.is_some() {
            query.push_str(", name = ? ");
        }
        if dto.parameters.is_some() {
            query.push_str(", parameters = ? ");
        }
        query.push_str("WHERE id = ?");

        let mut query_builder = sqlx::query(&query).bind(now);

        if let Some(name) = dto.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(parameters) = dto.parameters {
            let params_str = serde_json::to_string(&parameters)
                .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;
            query_builder = query_builder.bind(params_str);
        }

        query_builder = query_builder.bind(id);

        query_builder
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Self::find_by_id(id, pool).await
    }

    pub async fn delete(id: &str, pool: &Pool<Sqlite>) -> Result<()> {
        // Check if strategy exists
        Self::find_by_id(id, pool).await?;

        sqlx::query!("DELETE FROM strategies WHERE id = ?", id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            r#"
            CREATE TABLE strategies (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                strategy_type TEXT NOT NULL,
                parameters TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_find_strategy() {
        let pool = setup_test_db().await;
        let dto = CreateStrategyDto {
            name: "Test Strategy".to_string(),
            strategy_type: StrategyType::Classical,
            parameters: serde_json::json!({"period": 14}),
        };

        let created = Strategy::create(dto, &pool).await.unwrap();
        assert_eq!(created.name, "Test Strategy");
        assert_eq!(created.strategy_type, "classical");

        let found = Strategy::find_by_id(&created.id, &pool).await.unwrap();
        assert_eq!(found.name, "Test Strategy");
    }

    #[tokio::test]
    async fn test_update_strategy() {
        let pool = setup_test_db().await;
        let dto = CreateStrategyDto {
            name: "Old Name".to_string(),
            strategy_type: StrategyType::Classical,
            parameters: serde_json::json!({"val": 1}),
        };

        let created = Strategy::create(dto, &pool).await.unwrap();

        let update_dto = UpdateStrategyDto {
            name: Some("New Name".to_string()),
            parameters: Some(serde_json::json!({"val": 2})),
        };

        let updated = Strategy::update(&created.id, update_dto, &pool)
            .await
            .unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.parameters, r#"{"val":2}"#);
    }
}
