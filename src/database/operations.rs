use sqlx::PgPool;
use serenity::model::user::OnlineStatus;
use tracing::info;

use crate::database::models::{UserPresence, status_to_string};
use crate::Result;

/// Database operations for presence management
#[derive(Clone)]
pub struct DatabaseOperations {
    pool: PgPool,
}

impl DatabaseOperations {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get user presence by user and guild ID
    pub async fn get_user_presence(
        &self, 
        user_id: u64, 
        guild_id: u64
    ) -> Result<Option<UserPresence>> {
        let presence = sqlx::query_as::<_, UserPresence>(
            "SELECT * FROM user_presences WHERE user_id = $1 AND guild_id = $2"
        )
        .bind(user_id as i64)
        .bind(guild_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(presence)
    }

    /// Upsert user presence (simplified for single table)
    pub async fn upsert_presence(
        &self,
        user_id: u64,
        guild_id: u64,
        status: &OnlineStatus,
        activity_name: Option<String>,
        activity_type: Option<String>,
    ) -> Result<UserPresence> {
        let status_str = status_to_string(status);
        
        let presence = sqlx::query_as::<_, UserPresence>(
            r#"
            INSERT INTO user_presences 
                (user_id, guild_id, status, activity_name, activity_type, last_seen_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), NOW())
            ON CONFLICT (user_id, guild_id)
            DO UPDATE SET 
                status = $3,
                activity_name = $4,
                activity_type = $5,
                last_seen_at = NOW(),
                updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(user_id as i64)
        .bind(guild_id as i64)
        .bind(&status_str)
        .bind(activity_name)
        .bind(activity_type)
        .fetch_one(&self.pool)
        .await?;

        info!("Upserted presence: user_id={}, guild_id={}, status={:?}", user_id, guild_id, status);
        Ok(presence)
    }

    /// Get all presences with optional pagination
    pub async fn get_all_presences(
        &self, 
        limit: Option<i64>, 
        offset: Option<i64>
    ) -> Result<Vec<UserPresence>> {
        let mut query = "SELECT * FROM user_presences ORDER BY updated_at DESC".to_string();
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let presences = sqlx::query_as::<_, UserPresence>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(presences)
    }

    /// Get presences by guild ID
    pub async fn get_guild_presences(&self, guild_id: u64) -> Result<Vec<UserPresence>> {
        let presences = sqlx::query_as::<_, UserPresence>(
            "SELECT * FROM user_presences WHERE guild_id = $1 ORDER BY updated_at DESC"
        )
        .bind(guild_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(presences)
    }

    /// Get recent presences for analytics (using updated_at as history)
    pub async fn get_recent_presences(&self, hours: i32) -> Result<Vec<UserPresence>> {
        let presences = sqlx::query_as::<_, UserPresence>(
            "SELECT * FROM user_presences WHERE updated_at > NOW() - INTERVAL $1 HOUR ORDER BY updated_at DESC"
        )
        .bind(hours)
        .fetch_all(&self.pool)
        .await?;

        Ok(presences)
    }

    /// Clean old presence entries
    pub async fn cleanup_old_presences(&self, days: i32) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM user_presences WHERE last_seen_at < NOW() - INTERVAL $1 DAY"
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        info!("Cleaned up {} old presence entries", result.rows_affected());
        Ok(result.rows_affected())
    }
}
