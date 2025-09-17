use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

use crate::config::AppConfig;
use crate::Result;

/// Create and configure database connection pool
pub async fn create_db_pool(config: &AppConfig) -> Result<PgPool> {
    info!("Creating database connection pool...");
    
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    info!("Database pool created successfully");
    Ok(pool)
}
