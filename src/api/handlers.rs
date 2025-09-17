use warp::{Reply, Rejection};
use tracing::{info, error};

use crate::state::AppState;
use crate::api::errors::ApiError;
use crate::database::models::PresenceResponse;

/// Health check handler
pub async fn health_check_handler(app_state: AppState) -> Result<impl Reply, Rejection> {
    let is_healthy = app_state.health_check().await;
    
    if is_healthy {
        Ok(warp::reply::with_status("OK", warp::http::StatusCode::OK))
    } else {
        Err(warp::reject::custom(ApiError::InternalError("Service unhealthy".to_string())))
    }
}

/// Get all presences handler
pub async fn get_all_presences_handler(
    limit: Option<i64>,
    offset: Option<i64>,
    app_state: AppState,
) -> Result<impl Reply, Rejection> {
    match app_state.db.get_all_presences(limit, offset).await {
        Ok(presences) => {
            let responses: Vec<PresenceResponse> = presences
                .into_iter()
                .map(PresenceResponse::from)
                .collect();
            
            info!("[HTTP GET /presences] returning {} entries", responses.len());
            Ok(warp::reply::json(&responses))
        }
        Err(e) => {
            error!("Database error: {}", e);
            Err(warp::reject::custom(ApiError::DatabaseError(e.to_string())))
        }
    }
}

/// Get presence by user and guild ID handler
pub async fn get_presence_by_id_handler(
    user_id: u64,
    guild_id: u64,
    app_state: AppState,
) -> Result<impl Reply, Rejection> {
    // Try cache first
    match app_state.redis.get_cached_presence(user_id, guild_id).await {
        Ok(Some(presence)) => {
            info!("[HTTP GET /presences/{}/{}] found in cache", user_id, guild_id);
            return Ok(warp::reply::json(&PresenceResponse::from(presence)));
        }
        Ok(None) => {
            // Try database
            match app_state.db.get_user_presence(user_id, guild_id).await {
                Ok(Some(presence)) => {
                    info!("[HTTP GET /presences/{}/{}] found in database", user_id, guild_id);
                    
                    // Cache for next time
                    if let Err(e) = app_state.redis.cache_presence(user_id, guild_id, &presence).await {
                        error!("Failed to cache presence: {}", e);
                    }
                    
                    Ok(warp::reply::json(&PresenceResponse::from(presence)))
                }
                Ok(None) => {
                    info!("[HTTP GET /presences/{}/{}] not found", user_id, guild_id);
                    Err(warp::reject::custom(ApiError::NotFound))
                }
                Err(e) => {
                    error!("Database error: {}", e);
                    Err(warp::reject::custom(ApiError::DatabaseError(e.to_string())))
                }
            }
        }
        Err(e) => {
            error!("Cache error: {}", e);
            // Continue to database on cache error
            match app_state.db.get_user_presence(user_id, guild_id).await {
                Ok(Some(presence)) => Ok(warp::reply::json(&PresenceResponse::from(presence))),
                Ok(None) => Err(warp::reject::custom(ApiError::NotFound)),
                Err(e) => Err(warp::reject::custom(ApiError::DatabaseError(e.to_string()))),
            }
        }
    }
}

/// Get guild presences handler
pub async fn get_guild_presences_handler(
    guild_id: u64,
    app_state: AppState,
) -> Result<impl Reply, Rejection> {
    match app_state.db.get_guild_presences(guild_id).await {
        Ok(presences) => {
            let responses: Vec<PresenceResponse> = presences
                .into_iter()
                .map(PresenceResponse::from)
                .collect();
            
            info!("[HTTP GET /guilds/{}/presences] returning {} entries", guild_id, responses.len());
            Ok(warp::reply::json(&responses))
        }
        Err(e) => {
            error!("Database error: {}", e);
            Err(warp::reject::custom(ApiError::DatabaseError(e.to_string())))
        }
    }
}
