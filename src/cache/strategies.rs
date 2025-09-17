use crate::database::models::UserPresence;
use crate::database::operations::DatabaseOperations;
use crate::cache::RedisManager;
use crate::Result;

/// Different caching strategies for presence data
pub enum CacheStrategy {
    CacheAside,
    WriteThrough,
    WriteBack,
}

/// Cache-through pattern implementation
pub struct PresenceCache {
    redis: RedisManager,
    db: DatabaseOperations,
    strategy: CacheStrategy,
}

impl PresenceCache {
    pub fn new(redis: RedisManager, db: DatabaseOperations, strategy: CacheStrategy) -> Self {
        Self { redis, db, strategy }
    }

    /// Get presence using cache-aside pattern
    pub async fn get_presence(&self, user_id: u64) -> Result<Option<UserPresence>> {
        match self.strategy {
            CacheStrategy::CacheAside => {
                // Try cache first
                if let Some(presence) = self.redis.get_cached_presence(user_id).await? {
                    return Ok(Some(presence));
                }

                // Fallback to database
                if let Some(presence) = self.db.get_user_presence(user_id).await? {
                    // Cache the result
                    self.redis.cache_presence(user_id, &presence).await?;
                    Ok(Some(presence))
                } else {
                    Ok(None)
                }
            }
            _ => {
                // For other strategies, implement as needed
                self.db.get_user_presence(user_id).await
            }
        }
    }

    /// Set presence using selected strategy
    pub async fn set_presence(&self, presence: &UserPresence) -> Result<()> {
        let user_id = presence.user_id as u64;

        match self.strategy {
            CacheStrategy::WriteThrough => {
                // Write to both cache and database
                self.redis.cache_presence(user_id, presence).await?;
                // Database write would be handled by the calling code
                Ok(())
            }
            _ => {
                // Cache after database write
                self.redis.cache_presence(user_id, presence).await
            }
        }
    }
}
