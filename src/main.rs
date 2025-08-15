use std::sync::Arc;
use serenity::async_trait;
use serenity::model::{
    gateway::{GatewayIntents, Presence, Ready},
    user::OnlineStatus,
};
use serenity::prelude::*;
use dashmap::DashMap;
use warp::Filter;
use dotenv::dotenv;
use tokio::sync::oneshot;

type PresenceMap = Arc<DashMap<u64, OnlineStatus>>;

struct Handler {
    presences: PresenceMap,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot connected as {}", ready.user.name);

        // For each guild, save user_id and presence status from cache if available
        for guild_id in ctx.cache.guilds() {
            if let Some(guild_data) = ctx.cache.guild(guild_id) {
                for (user_id, presence) in &guild_data.presences {
                    let user_id = user_id.get();
                    let status = presence.status;
                    self.presences.insert(user_id, status);
                    println!(
                        "[READY] Inserted: user_id = {}, status = {:?}",
                        user_id, status
                    );
                }
            }
        }
        println!(
            "[READY] Initial population complete: {} entries in DashMap",
            self.presences.len()
        );
    }

    async fn presence_update(&self, _ctx: Context, new_data: Presence) {
        let user_id = new_data.user.id.get();
        let new_status = new_data.status;
        println!(
            "Received presence_update: user_id = {}, status = {:?}",
            user_id, new_status
        );
        self.presences.insert(user_id, new_status);
        println!("Updated presence map: now {} entries", self.presences.len());
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");

    let presence_map: PresenceMap = Arc::new(DashMap::new());
    let handler = Handler {
        presences: Arc::clone(&presence_map),
    };

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    let http_presence_map = Arc::clone(&presence_map);

    let all_presences = {
        let presences = Arc::clone(&http_presence_map);
        warp::path!("presences")
            .and(warp::get())
            .map(move || {
                let data: Vec<_> = presences
                    .iter()
                    .map(|entry| (entry.key().to_string(), format!("{:?}", entry.value())))
                    .collect();
                println!("[HTTP GET /presences] returning {} entries: {:?}", data.len(), data);
                warp::reply::json(&data)
            })
    };
    let presence_by_id = warp::path!("presences" / u64)
        .and(warp::get())
        .and(warp::any().map(move || Arc::clone(&http_presence_map)))
        .map(|user_id: u64, presences: PresenceMap| {
            if let Some(status) = presences.get(&user_id) {
                println!("[HTTP GET /presences/{}] found status {:?}", user_id, status);
                warp::reply::with_status(
                    format!("{:?}", *status),
                    warp::http::StatusCode::OK,
                )
            } else {
                println!("[HTTP GET /presences/{}] not found", user_id);
                warp::reply::with_status(
                    "Not found".to_string(),
                    warp::http::StatusCode::NOT_FOUND,
                )
            }
        });

    let routes = all_presences.or(presence_by_id);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async {
            shutdown_rx.await.ok();
        });
    println!("Starting HTTP server on {}...", addr);
    let warp_handle = tokio::spawn(server);

    // Spawn the client in a background task
    let client_handle = tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    });

    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
    println!("Shutdown signal received, stopping...");

    let _ = shutdown_tx.send(());
    let _ = warp_handle.await;
    
    // Shutdown shards gracefully
    client_handle.abort(); // Abort client task on shutdown

    println!("Shutdown complete.");
}
