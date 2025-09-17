use serenity::async_trait;
use serenity::model::{
    gateway::{Presence, Ready},
};
use serenity::prelude::*;
use tracing::info;

use crate::state::AppState;

/// Discord bot event handler
pub struct BotHandler {
    pub app_state: AppState,
}

impl BotHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl EventHandler for BotHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Bot connected as {} (ID: {})", ready.user.name, ready.user.id);
        info!("Bot is in {} guilds", ready.guilds.len());

        // Populate initial presence data from Discord cache and save to database
        let mut total_presences = 0;
        
        // Extract presence data first to avoid Send issues
        let mut presence_data = Vec::new();
        for guild_id in ctx.cache.guilds() {
            let guild_id_u64 = guild_id.get();
            if let Some(guild_data) = ctx.cache.guild(guild_id) {
                for (user_id, presence) in &guild_data.presences {
                    let user_id = user_id.get();
                    let status = presence.status;
                    presence_data.push((user_id, guild_id_u64, status));
                }
            }
        }
        
        // Now process the extracted data
        for (user_id, guild_id_u64, status) in presence_data {
            // Save to database instead of DashMap
            match self.app_state.db.upsert_presence(
                user_id, 
                guild_id_u64, 
                &status,
                None, // activity_name
                None, // activity_type
            ).await {
                Ok(presence_record) => {
                    // Cache in Redis for fast access
                    if let Err(e) = self.app_state.redis.cache_presence(user_id, guild_id_u64, &presence_record).await {
                        tracing::error!("Failed to cache presence: {}", e);
                    }
                    total_presences += 1;
                    
                    // Log only first few entries to avoid spam
                    if total_presences <= 10 {
                        info!("[READY] Saved: user_id = {}, status = {:?}", user_id, status);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to save presence for user {}: {}", user_id, e);
                }
            }
        }
        info!("[READY] Initial population complete: {} entries saved to database", total_presences);
    }

    async fn presence_update(&self, _ctx: Context, new_data: Presence) {
        let user_id = new_data.user.id.get();
        let new_status = new_data.status;
        
        info!("Received presence_update: user_id = {}, status = {:?}", user_id, new_status);
        
        // Since we don't have guild_id from presence data, we'll need to get it from the first guild
        // In a production bot, you'd want to handle multiple guilds properly
        let guild_id = 1; // Default guild ID or extract from context
        
        // Get old status from Redis first, then database if not found
        let old_status = match self.app_state.redis.get_cached_presence(user_id, guild_id).await {
            Ok(Some(presence)) => Some(presence.status),
            _ => {
                // Try database if not in cache
                match self.app_state.db.get_user_presence(user_id, guild_id).await {
                    Ok(Some(presence)) => Some(presence.status),
                    _ => None
                }
            }
        };
        
        // Save to database
        match self.app_state.db.upsert_presence(
            user_id, 
            guild_id, 
            &new_status,
            None, // Could extract from new_data.activities
            None, // activity_type (now String)
        ).await {
            Ok(presence_record) => {
                // Cache in Redis
                if let Err(e) = self.app_state.redis.cache_presence(user_id, guild_id, &presence_record).await {
                    tracing::error!("Failed to cache updated presence: {}", e);
                }
                
                // Log status changes for monitoring
                match old_status {
                    Some(old) if old != crate::database::models::status_to_string(&new_status) => {
                        info!("Status changed for user {}: {} -> {:?}", user_id, old, new_status);
                    }
                    None => {
                        info!("New user presence tracked: {} -> {:?}", user_id, new_status);
                    }
                    _ => {} // No change, don't log
                }
                
                info!("Updated presence in database and cache for user {}", user_id);
            }
            Err(e) => {
                tracing::error!("Failed to update presence for user {}: {}", user_id, e);
            }
        }
    }
}
