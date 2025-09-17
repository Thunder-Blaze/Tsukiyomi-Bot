use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::model::user::OnlineStatus;
use sqlx::FromRow;

/// User presence data model (simplified to single table)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPresence {
    pub id: i64,
    pub user_id: i64,
    pub status: String, // We'll convert OnlineStatus to string
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response model for API
#[derive(Debug, Serialize)]
pub struct PresenceResponse {
    pub user_id: String,
    pub status: String,
    pub last_seen_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserPresence> for PresenceResponse {
    fn from(presence: UserPresence) -> Self {
        Self {
            user_id: presence.user_id.to_string(),
            status: presence.status,
            last_seen_at: presence.last_seen_at,
            updated_at: presence.updated_at,
        }
    }
}

/// Helper function to convert OnlineStatus to string for database
pub fn status_to_string(status: &OnlineStatus) -> String {
    match status {
        OnlineStatus::Online => "Online".to_string(),
        OnlineStatus::Idle => "Idle".to_string(),
        OnlineStatus::DoNotDisturb => "Do Not Disturb".to_string(),
        OnlineStatus::Invisible => "Invisible".to_string(),
        OnlineStatus::Offline => "Offline".to_string(),
        _ => "Offline".to_string(), // Default for unknown statuses
    }
}
