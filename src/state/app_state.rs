use crate::database::operations::DatabaseOperations;
use crate::cache::RedisManager;
use crate::config::AppConfig;
use crate::database::pool::create_db_pool;
use crate::Result;

/// Shared application state (Redis + PostgreSQL only)
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseOperations,
    pub redis: RedisManager,
}

impl AppState {
    /// Initialize application state
    pub async fn new(config: &AppConfig) -> Result<Self> {
        // Initialize database
        let db_pool = create_db_pool(config).await?;
        let db = DatabaseOperations::new(db_pool);

        // Initialize Redis
        let redis = RedisManager::new(config).await?;

        Ok(Self {
            db,
            redis,
        })
    }

    /// Health check for all services
    pub async fn health_check(&self) -> bool {
        // Check Redis
        let redis_healthy = self.redis.health_check().await;
        
        // Check database (simple query)
        let db_healthy = self.db.get_all_presences(Some(1), None).await.is_ok();

        redis_healthy && db_healthy
    }
}
