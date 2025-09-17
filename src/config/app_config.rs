use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub discord_token: String,
    pub port: u16,
    pub redis_pool_max_open: u64,
    pub redis_pool_max_idle: u64,
    pub log_level: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            discord_token: env::var("DISCORD_TOKEN")?,
            port: env::var("PORT")?.parse()?,
            redis_pool_max_open: env::var("REDIS_POOL_MAX_OPEN")
                .unwrap_or_else(|_| "16".to_string())
                .parse()?,
            redis_pool_max_idle: env::var("REDIS_POOL_MAX_IDLE")
                .unwrap_or_else(|_| "8".to_string())
                .parse()?,
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            database_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()?,
            database_min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL cannot be empty".into());
        }
        if self.discord_token.is_empty() {
            return Err("DISCORD_TOKEN cannot be empty".into());
        }
        if self.port == 0 {
            return Err("PORT must be a valid port number".into());
        }
        Ok(())
    }
}
