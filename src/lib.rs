//! Production-ready Discord bot with PostgreSQL and Redis
//! 
//! This crate provides a modular architecture for a Discord presence tracking bot
//! with database persistence and caching capabilities.

pub mod config;
pub mod database;
pub mod cache;
pub mod bot;
pub mod api;
pub mod state;
pub mod utils;

// Re-export commonly used types
pub use state::AppState;
pub use config::AppConfig;
pub use database::models::*;
pub use api::errors::ApiError;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
