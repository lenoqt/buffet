use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::User;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub user_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateItemDto {
    pub name: String,
    pub description: Option<String>,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateItemDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Item {
    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>("SELECT * FROM items ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(items)
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<Item> {
        let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Item with ID {} not found", id)))?;

        Ok(item)
    }

    pub async fn find_by_user_id(user_id: &str, pool: &Pool<Sqlite>) -> Result<Vec<Item>> {
        // Verify user exists
        User::find_by_id(user_id, pool).await?;

        let items = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(items)
    }

    pub async fn create(dto: CreateItemDto, pool: &Pool<Sqlite>) -> Result<Item> {
        // Verify user exists
        User::find_by_id(&dto.user_id, pool).await?;

        let id = Uuid::new_v4().to_string();
        let now = OffsetDateTime::now_utc();

        // Insert the new item
        sqlx::query!(
            r#"
            INSERT INTO items (id, name, description, user_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id,
            dto.name,
            dto.description,
            dto.user_id,
            now,
            now
        )
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        // Fetch the created item
        Self::find_by_id(&id, pool).await
    }

    pub async fn update(id: &str, dto: UpdateItemDto, pool: &Pool<Sqlite>) -> Result<Item> {
        // Check if item exists
        let _item = Self::find_by_id(id, pool).await?;

        // Build update query dynamically based on provided fields
        let mut query = String::from("UPDATE items SET updated_at = ? ");
        let now = OffsetDateTime::now_utc();

        // Add fields to update if they are provided
        if dto.name.is_some() {
            query.push_str(", name = ? ");
        }
        if dto.description.is_some() {
            query.push_str(", description = ? ");
        }
        query.push_str("WHERE id = ?");

        // Build query with arguments
        let mut query_builder = sqlx::query(&query)
            .bind(now);

        if let Some(name) = dto.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(description) = dto.description {
            query_builder = query_builder.bind(description);
        }

        query_builder = query_builder.bind(id);

        // Execute the update
        query_builder
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        // Fetch and return the updated item
        Self::find_by_id(id, pool).await
    }

    pub async fn delete(id: &str, pool: &Pool<Sqlite>) -> Result<()> {
        // Check if item exists
        let _item = Self::find_by_id(id, pool).await?;

        // Delete the item
        sqlx::query!("DELETE FROM items WHERE id = ?", id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }
}