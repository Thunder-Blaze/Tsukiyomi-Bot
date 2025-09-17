use serenity::model::{gateway::Presence, user::OnlineStatus};
use serenity::prelude::Context;
use tracing::{info, error};

use crate::state::AppState;
use crate::database::models::status_to_string;

/// Handle bot ready event with extracted data (thread-safe)
pub async fn handle_ready_event_with_data(initial_presences: Vec<(u64, OnlineStatus)>, app_state: AppState) {
    info!("Processing ready event - populating initial presence data for {} presences", initial_presences.len());
    
    let mut successful_saves = 0;
    
    // Process presences in batches to avoid overwhelming the database
    for chunk in initial_presences.chunks(50) {
        let mut tasks = vec![];
        
        for &(user_id, status) in chunk {
            let app_state = app_state.clone();
            
            let task = tokio::spawn(async move {
                match app_state.db.upsert_presence(
                    user_id, 
                    &status,
                ).await {
                    Ok(presence) => {
                        // Cache the presence
                        if let Err(e) = app_state.redis.cache_presence(user_id, &presence).await {
                            error!("Failed to cache presence: {}", e);
                        }
                        true
                    }
                    Err(e) => {
                        error!("Failed to save presence for user {}: {}", user_id, e);
                        false
                    }
                }
            });
            
            tasks.push(task);
        }
        
        // Wait for batch completion
        for task in tasks {
            if let Ok(success) = task.await {
                if success {
                    successful_saves += 1;
                }
            }
        }
    }
    
    info!("[READY] Initial population complete: {} presences saved to database", successful_saves);
}

/// Handle presence update with database storage
pub async fn handle_presence_update(_ctx: Context, presence: Presence, app_state: AppState) {
    let user_id = presence.user.id.get();
    let new_status = presence.status;
    
    info!("Received presence_update: user_id = {}, status = {:?}", user_id, new_status);
    
    // Get old status from Redis cache first, then database if not found
    let old_status = match app_state.redis.get_cached_presence(user_id).await {
        Ok(Some(presence)) => Some(presence.status),
        _ => {
            // Try database if not in cache
            match app_state.db.get_user_presence(user_id).await {
                Ok(Some(presence)) => Some(presence.status),
                _ => None
            }
        }
    };
    
    // Save to database
    match app_state.db.upsert_presence(
        user_id, 
        &new_status,
    ).await {
        Ok(presence_record) => {
            // Cache the updated presence
            if let Err(e) = app_state.redis.cache_presence(user_id, &presence_record).await {
                error!("Failed to cache updated presence: {}", e);
            }
            
            // Log status changes for monitoring
            let new_status_str = status_to_string(&new_status);
            match old_status {
                Some(ref old) if old != &new_status_str => {
                    info!("Status changed for user {}: {} -> {}", user_id, old, new_status_str);
                }
                None => {
                    info!("New user presence tracked: {} -> {}", user_id, new_status_str);
                }
                _ => {} // No change, don't log
            }
            
            info!("Updated presence in database and cache for user {}", user_id);
        }
        Err(e) => {
            error!("Failed to update presence for user {}: {}", user_id, e);
        }
    }
}
