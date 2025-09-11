use dashmap::DashMap;
use dotenv::dotenv;
use serenity::async_trait;
use serenity::model::{
    gateway::{GatewayIntents, Presence, Ready},
    user::OnlineStatus,
};
use serenity::prelude::*;
use std::sync::Arc;
use tokio::sync::oneshot;
use std::env;
use warp::Filter;

type PresenceMap = Arc<DashMap<u64, OnlineStatus>>;

struct Handler {
    presences: PresenceMap,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot connected as {}", ready.user.name);

        for guild_id in ctx.cache.guilds() {
            if let Some(guild_data) = ctx.cache.guild(guild_id) {
                for (user_id, presence) in &guild_data.presences {
                    let user_id = user_id.get();
                    let status = presence.status;
                    self.presences.insert(user_id, status);
                    println!("[READY] Inserted: user_id = {}, status = {:?}", user_id, status);
                }
            }
        }
        println!("[READY] Initial population complete: {} entries in DashMap", self.presences.len());
    }

    async fn presence_update(&self, _ctx: Context, new_data: Presence) {
        let user_id = new_data.user.id.get();
        let new_status = new_data.status;
        println!("Received presence_update: user_id = {}, status = {:?}", user_id, new_status);
        self.presences.insert(user_id, new_status);
        println!("Updated presence map: now {} entries", self.presences.len());
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");

    let port_env = env::var("PORT").expect("PORT must be set");
    let port: u16 = port_env.parse().expect("PORT must be a valid u16");
    println!("Render-provided PORT env var: '{}', parsed as {}", port_env, port);

    let presence_map: PresenceMap = Arc::new(DashMap::new());
    let handler = Handler {
        presences: Arc::clone(&presence_map),
    };

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;

    let health_check = warp::path::end()
    .and(warp::method())
    .and_then(|method: warp::http::Method| async move {
        if method == warp::http::Method::HEAD || method == warp::http::Method::GET {
            Ok::<_, warp::Rejection>(warp::reply::with_status("OK", warp::http::StatusCode::OK))
        } else {
            Err(warp::reject::not_found())
        }
    });

    let http_presence_map = Arc::clone(&presence_map);
    let all_presences = {
        let presences = Arc::clone(&http_presence_map);
        warp::path!("presences").and(warp::get()).map(move || {
            let data: Vec<_> = presences
                .iter()
                .map(|entry| (entry.key().to_string(), format!("{:?}", entry.value())))
                .collect();
            println!(
                "[HTTP GET /presences] returning {} entries: {:?}",
                data.len(), data
            );
            warp::reply::json(&data)
        })
    };

    let presence_by_id = warp::path!("presences" / u64)
        .and(warp::get())
        .and(warp::any().map(move || Arc::clone(&http_presence_map)))
        .map(|user_id: u64, presences: PresenceMap| {
            if let Some(status) = presences.get(&user_id) {
                println!("[HTTP GET /presences/{}] found status {:?}", user_id, status);
                warp::reply::with_status(format!("{:?}", *status), warp::http::StatusCode::OK)
            } else {
                println!("[HTTP GET /presences/{}] not found", user_id);
                warp::reply::with_status("NotFound".to_string(), warp::http::StatusCode::OK)
            }
        });

    let routes = health_check.or(all_presences).or(presence_by_id);

    let (_shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Start the HTTP server and bind to $PORT in the foreground
    let (addr, server_future) = warp::serve(routes)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
            shutdown_rx.await.ok();
        });

    println!("Starting HTTP server on {}...", addr);

    // Start the bot as a background task
    let serenity_token = token.clone();
    let mut serenity_client = serenity::Client::builder(serenity_token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    let bot_handle = tokio::spawn(async move {
        if let Err(why) = serenity_client.start().await {
            println!("Client error: {:?}", why);
        }
    });

    // Await the warp HTTP server in the foreground (critical for Render)
    server_future.await;

    // Cleanup
    bot_handle.abort();
    println!("Shutdown complete.");
}
