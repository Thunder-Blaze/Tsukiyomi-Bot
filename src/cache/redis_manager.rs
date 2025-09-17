use mobc::{Pool, Connection};
use mobc_redis::{RedisConnectionManager, redis};
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{info, error, warn};

use crate::config::AppConfig;
use crate::database::models::UserPresence;
use crate::Result;

pub type RedisPool = Pool<RedisConnectionManager>;
pub type RedisConnection = Connection<RedisConnectionManager>;

/// Redis cache manager
#[derive(Clone)]
pub struct RedisManager {
    pool: RedisPool,
}

impl RedisManager {
    /// Create new Redis manager
    pub async fn new(config: &AppConfig) -> Result<Self> {
        info!("Initializing Redis connection pool...");
        
        let redis_client = redis::Client::open(config.redis_url.as_str())?;
        let redis_manager = RedisConnectionManager::new(redis_client);
        let pool = Pool::builder()
            .max_open(config.redis_pool_max_open)
            .max_idle(config.redis_pool_max_idle)
            .build(redis_manager);

        Ok(Self { pool })
    }

    /// Get connection from pool
    async fn get_connection(&self) -> Result<RedisConnection> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                Err(e.into())
            }
        }
    }

    /// Cache user presence
    pub async fn cache_presence(&self, user_id: u64, guild_id: u64, presence: &UserPresence) -> Result<()> {
        let key = format!("presence:{}:{}", user_id, guild_id);
        let data = serde_json::to_string(presence)?;
        
        let mut conn = self.get_connection().await?;
        // TTL in seconds for Redis compatibility
        let ttl: u64 = 300; // 5 minutes TTL
        let _: () = conn.set_ex(&key, &data, ttl).await?;
        
        Ok(())
    }

    /// Get cached presence
    pub async fn get_cached_presence(&self, user_id: u64, guild_id: u64) -> Result<Option<UserPresence>> {
        let key = format!("presence:{}:{}", user_id, guild_id);
        
        let mut conn = self.get_connection().await?;
        // Use redis::Value to handle nil responses properly
        match conn.get::<_, redis::Value>(&key).await {
            Ok(redis::Value::BulkString(data)) => {
                // Convert bytes to string
                let data_str = String::from_utf8(data).unwrap_or_default();
                match serde_json::from_str::<UserPresence>(&data_str) {
                    Ok(presence) => Ok(Some(presence)),
                    Err(e) => {
                        warn!("Failed to deserialize cached presence: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(redis::Value::Nil) => Ok(None), // Handle nil response
            Ok(_) => {
                warn!("Unexpected Redis value type for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                warn!("Redis error for key {}: {}", key, e);
                Ok(None)
            }
        }
    }

    /// Invalidate presence cache
    pub async fn invalidate_presence(&self, user_id: u64, guild_id: u64) -> Result<()> {
        let key = format!("presence:{}:{}", user_id, guild_id);
        
        let mut conn = self.get_connection().await?;
        // Use proper error handling instead of unwrap_or
        match conn.del::<_, i32>(&key).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to delete Redis key {}: {}", key, e);
                Err(e.into())
            }
        }
    }

    /// Cache generic data with custom TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: u64) -> Result<()> {
        let data = serde_json::to_string(value)?;
        let mut conn = self.get_connection().await?;
        // TTL should be u64 for the latest Redis version
        let _: () = conn.set_ex(key, &data, ttl).await?;
        Ok(())
    }

    /// Get generic cached data
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.get_connection().await?;
        // Use redis::Value to handle nil responses properly
        match conn.get::<_, redis::Value>(key).await {
            Ok(redis::Value::BulkString(data)) => {
                // Convert bytes to string
                let data_str = String::from_utf8(data).unwrap_or_default();
                match serde_json::from_str::<T>(&data_str) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        warn!("Failed to deserialize cached data for key {}: {}", key, e);
                        Ok(None)
                    }
                }
            }
            Ok(redis::Value::Nil) => Ok(None), // Handle nil response
            Ok(_) => {
                warn!("Unexpected Redis value type for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                warn!("Redis error for key {}: {}", key, e);
                Ok(None)
            }
        }
    }

    /// Delete cached data
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        match conn.del::<_, i32>(key).await {
            Ok(count) => Ok(count > 0), // Returns true if at least one key was deleted
            Err(e) => {
                error!("Failed to delete Redis key {}: {}", key, e);
                Ok(false) // Return false instead of propagating error for this method
            }
        }
    }

    /// Check if Redis is healthy
    pub async fn health_check(&self) -> bool {
        match self.get_connection().await {
            Ok(mut conn) => {
                // Use AsyncCommands ping method
                match conn.get::<_, Option<String>>("__ping_test__").await {
                    Ok(_) => true, // Connection works
                    Err(e) => {
                        error!("Redis health check failed: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Redis connection for health check: {}", e);
                false
            }
        }
    }
}
