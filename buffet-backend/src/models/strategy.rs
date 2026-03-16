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
    pub status: String,        // "active", "inactive", "error"
    pub symbols: String,       // JSON array string e.g. '["AAPL","GOOG"]'
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

/// Strategy lifecycle status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyStatus {
    Active,
    Inactive,
    Error,
}

impl std::fmt::Display for StrategyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyStatus::Active => write!(f, "active"),
            StrategyStatus::Inactive => write!(f, "inactive"),
            StrategyStatus::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for StrategyStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(StrategyStatus::Active),
            "inactive" => Ok(StrategyStatus::Inactive),
            "error" => Ok(StrategyStatus::Error),
            _ => Err(format!("Unknown strategy status: {}", s)),
        }
    }
}

/// DTO for creating a new strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStrategyDto {
    pub name: String,
    pub strategy_type: StrategyType,
    pub parameters: serde_json::Value,
    pub status: Option<String>,
    pub symbols: Option<Vec<String>>,
}

/// DTO for updating a strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStrategyDto {
    pub name: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub symbols: Option<Vec<String>>,
}

/// Validate that the provided parameters are valid for the given strategy type.
///
/// - `Classical`: requires `fast_period` and `slow_period` as positive integers,
///   with `fast_period < slow_period`.
/// - `Statistical` / `MLBased`: pass-through (no validation for now).
pub fn validate_parameters(
    strategy_type: &StrategyType,
    params: &serde_json::Value,
) -> std::result::Result<(), String> {
    match strategy_type {
        StrategyType::Classical => {
            let fast = params.get("fast_period").ok_or_else(|| {
                "Classical strategy requires 'fast_period' parameter".to_string()
            })?;
            let slow = params.get("slow_period").ok_or_else(|| {
                "Classical strategy requires 'slow_period' parameter".to_string()
            })?;

            let fast_val = fast
                .as_u64()
                .filter(|&v| v > 0)
                .ok_or_else(|| "'fast_period' must be a positive integer".to_string())?;

            let slow_val = slow
                .as_u64()
                .filter(|&v| v > 0)
                .ok_or_else(|| "'slow_period' must be a positive integer".to_string())?;

            if fast_val >= slow_val {
                return Err(format!(
                    "'fast_period' ({}) must be less than 'slow_period' ({})",
                    fast_val, slow_val
                ));
            }

            Ok(())
        }
        // Pass-through for Statistical and MLBased
        StrategyType::Statistical | StrategyType::MLBased => Ok(()),
    }
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

    /// Find all strategies with a given status string (e.g. "active", "inactive", "error").
    pub async fn find_by_status(status: &str, pool: &Pool<Sqlite>) -> Result<Vec<Strategy>> {
        let strategies = sqlx::query_as::<_, Strategy>(
            "SELECT * FROM strategies WHERE status = ? ORDER BY created_at DESC",
        )
        .bind(status)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(strategies)
    }

    pub async fn create(dto: CreateStrategyDto, pool: &Pool<Sqlite>) -> Result<Strategy> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let strategy_type = dto.strategy_type.to_string();
        let parameters = serde_json::to_string(&dto.parameters)
            .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;

        let status = dto
            .status
            .unwrap_or_else(|| StrategyStatus::Inactive.to_string());

        let symbols_vec = dto.symbols.unwrap_or_default();
        let symbols = serde_json::to_string(&symbols_vec)
            .map_err(|e| AppError::BadRequest(format!("Invalid symbols: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO strategies (id, name, strategy_type, parameters, created_at, updated_at, status, symbols)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            dto.name,
            strategy_type,
            parameters,
            now,
            now,
            status,
            symbols
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
        let mut query = String::from("UPDATE strategies SET updated_at = ?");

        if dto.name.is_some() {
            query.push_str(", name = ?");
        }
        if dto.parameters.is_some() {
            query.push_str(", parameters = ?");
        }
        if dto.symbols.is_some() {
            query.push_str(", symbols = ?");
        }
        query.push_str(" WHERE id = ?");

        let mut query_builder = sqlx::query(&query).bind(now);

        if let Some(name) = dto.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(parameters) = dto.parameters {
            let params_str = serde_json::to_string(&parameters)
                .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;
            query_builder = query_builder.bind(params_str);
        }
        if let Some(symbols) = dto.symbols {
            let symbols_str = serde_json::to_string(&symbols)
                .map_err(|e| AppError::BadRequest(format!("Invalid symbols: {}", e)))?;
            query_builder = query_builder.bind(symbols_str);
        }

        query_builder = query_builder.bind(id);

        query_builder
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Self::find_by_id(id, pool).await
    }

    /// Set the `status` of a strategy by ID and return the updated record.
    pub async fn set_status(
        id: &str,
        status: StrategyStatus,
        pool: &Pool<Sqlite>,
    ) -> Result<Strategy> {
        // Verify the strategy exists first
        Self::find_by_id(id, pool).await?;

        let now = Utc::now();
        let status_str = status.to_string();

        sqlx::query!(
            "UPDATE strategies SET status = ?, updated_at = ? WHERE id = ?",
            status_str,
            now,
            id
        )
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
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL DEFAULT 'inactive',
                symbols TEXT NOT NULL DEFAULT '[]'
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
            parameters: serde_json::json!({"fast_period": 5, "slow_period": 20}),
            status: None,
            symbols: None,
        };

        let created = Strategy::create(dto, &pool).await.unwrap();
        assert_eq!(created.name, "Test Strategy");
        assert_eq!(created.strategy_type, "classical");
        assert_eq!(created.status, "inactive");
        assert_eq!(created.symbols, "[]");

        let found = Strategy::find_by_id(&created.id, &pool).await.unwrap();
        assert_eq!(found.name, "Test Strategy");
    }

    #[tokio::test]
    async fn test_update_strategy() {
        let pool = setup_test_db().await;
        let dto = CreateStrategyDto {
            name: "Old Name".to_string(),
            strategy_type: StrategyType::Classical,
            parameters: serde_json::json!({"fast_period": 5, "slow_period": 20}),
            status: None,
            symbols: None,
        };

        let created = Strategy::create(dto, &pool).await.unwrap();

        let update_dto = UpdateStrategyDto {
            name: Some("New Name".to_string()),
            parameters: Some(serde_json::json!({"fast_period": 5, "slow_period": 20})),
            symbols: Some(vec!["AAPL".to_string(), "GOOG".to_string()]),
        };

        let updated = Strategy::update(&created.id, update_dto, &pool)
            .await
            .unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.symbols, r#"["AAPL","GOOG"]"#);
    }

    #[tokio::test]
    async fn test_set_status() {
        let pool = setup_test_db().await;
        let dto = CreateStrategyDto {
            name: "Status Test".to_string(),
            strategy_type: StrategyType::Statistical,
            parameters: serde_json::json!({}),
            status: None,
            symbols: None,
        };

        let created = Strategy::create(dto, &pool).await.unwrap();
        assert_eq!(created.status, "inactive");

        let activated = Strategy::set_status(&created.id, StrategyStatus::Active, &pool)
            .await
            .unwrap();
        assert_eq!(activated.status, "active");

        let errored = Strategy::set_status(&created.id, StrategyStatus::Error, &pool)
            .await
            .unwrap();
        assert_eq!(errored.status, "error");
    }

    #[tokio::test]
    async fn test_find_by_status() {
        let pool = setup_test_db().await;

        for (name, status) in [
            ("S1", Some("active".to_string())),
            ("S2", Some("active".to_string())),
            ("S3", None),
        ] {
            Strategy::create(
                CreateStrategyDto {
                    name: name.to_string(),
                    strategy_type: StrategyType::Statistical,
                    parameters: serde_json::json!({}),
                    status,
                    symbols: None,
                },
                &pool,
            )
            .await
            .unwrap();
        }

        let active = Strategy::find_by_status("active", &pool).await.unwrap();
        assert_eq!(active.len(), 2);

        let inactive = Strategy::find_by_status("inactive", &pool).await.unwrap();
        assert_eq!(inactive.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_parameters_classical_valid() {
        let params = serde_json::json!({"fast_period": 5, "slow_period": 20});
        assert!(validate_parameters(&StrategyType::Classical, &params).is_ok());
    }

    #[tokio::test]
    async fn test_validate_parameters_classical_missing_fast() {
        let params = serde_json::json!({"slow_period": 20});
        let err = validate_parameters(&StrategyType::Classical, &params).unwrap_err();
        assert!(err.contains("fast_period"));
    }

    #[tokio::test]
    async fn test_validate_parameters_classical_fast_gte_slow() {
        let params = serde_json::json!({"fast_period": 20, "slow_period": 5});
        let err = validate_parameters(&StrategyType::Classical, &params).unwrap_err();
        assert!(err.contains("less than"));
    }

    #[tokio::test]
    async fn test_validate_parameters_statistical_passthrough() {
        let params = serde_json::json!({"anything": "goes"});
        assert!(validate_parameters(&StrategyType::Statistical, &params).is_ok());
    }

    #[tokio::test]
    async fn test_validate_parameters_ml_passthrough() {
        let params = serde_json::json!({});
        assert!(validate_parameters(&StrategyType::MLBased, &params).is_ok());
    }
}
