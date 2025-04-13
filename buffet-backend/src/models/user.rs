use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

impl User {
    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(users)
    }

    pub async fn find_by_id(id: &str, pool: &Pool<Sqlite>) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", id)))?;

        Ok(user)
    }

    #[allow(dead_code)]
    pub async fn find_by_username(username: &str, pool: &Pool<Sqlite>) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("User with username {} not found", username)))?;

        Ok(user)
    }

    pub async fn create(dto: CreateUserDto, pool: &Pool<Sqlite>) -> Result<User> {
        // In a real application, you would hash the password here
        // For this example, we'll store it as plain text (NOT recommended for production)
        let password_hash = dto.password;

        let id = Uuid::new_v4().to_string();
        let now = OffsetDateTime::now_utc();

        // Check if username or email already exists
        let existing = sqlx::query!(
            "SELECT id FROM users WHERE username = ? OR email = ?",
            dto.username,
            dto.email
        )
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Username or email already exists".to_string()));
        }

        // Insert the new user
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at) 
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id,
            dto.username,
            dto.email,
            password_hash,
            now,
            now
        )
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        // Fetch the created user
        Self::find_by_id(&id, pool).await
    }

    pub async fn update(id: &str, dto: UpdateUserDto, pool: &Pool<Sqlite>) -> Result<User> {
        // Check if user exists
        Self::find_by_id(id, pool).await?;

        // Build update query dynamically based on provided fields
        let mut query = String::from("UPDATE users SET updated_at = ? ");
        let now = OffsetDateTime::now_utc();

        // Add fields to update if they are provided
        if dto.username.is_some() {
            query.push_str(", username = ? ");
        }
        if dto.email.is_some() {
            query.push_str(", email = ? ");
        }
        if dto.password.is_some() {
            query.push_str(", password_hash = ? ");
        }
        query.push_str("WHERE id = ?");

        // Build query with arguments
        let mut query_builder = sqlx::query(&query)
            .bind(now);

        if let Some(username) = dto.username {
            query_builder = query_builder.bind(username);
        }
        if let Some(email) = dto.email {
            query_builder = query_builder.bind(email);
        }
        if let Some(password) = dto.password {
            // In a real application, you would hash the password here
            query_builder = query_builder.bind(password);
        }

        query_builder = query_builder.bind(id);

        // Execute the update
        query_builder
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        // Fetch and return the updated user
        Self::find_by_id(id, pool).await
    }

    pub async fn delete(id: &str, pool: &Pool<Sqlite>) -> Result<()> {
        // Check if user exists
        let _user = Self::find_by_id(id, pool).await?;

        // Delete the user
        sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }
}