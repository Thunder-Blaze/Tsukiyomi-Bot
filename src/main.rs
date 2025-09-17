use dotenv::dotenv;
use serenity::{
    all::GatewayIntents,
    Client,
};
use tokio::sync::oneshot;
use tracing::info;

use tsukiyomi_bot::{
    config::AppConfig,
    state::AppState,
    bot::BotHandler,
    api::create_routes,
    utils::setup_logging,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    setup_logging().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
        format!("Failed to setup logging: {}", e).into()
    })?;
    
    // Load configuration
    dotenv().ok();
    let config = AppConfig::from_env().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
        format!("Failed to load config: {}", e).into()
    })?;
    config.validate().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
        format!("Config validation failed: {}", e).into()
    })?;
    
    info!("Starting Discord bot with port: {}", config.port);

    // Initialize application state
    let app_state = AppState::new(&config).await?;
    info!("Application state initialized successfully");

    // Create bot handler
    let handler = BotHandler::new(app_state.clone());

    // Setup Discord bot
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;

    // Create API routes
    let routes = create_routes(app_state);

    // Graceful shutdown
    let (_shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Start HTTP server
    let (addr, server_future) = warp::serve(routes)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], config.port), async {
            shutdown_rx.await.ok();
        });

    info!("HTTP server starting on {}", addr);

    // Start Discord bot
    let mut serenity_client = Client::builder(config.discord_token.clone(), intents)
        .event_handler(handler)
        .await?;

    let bot_handle = tokio::spawn(async move {
        if let Err(why) = serenity_client.start().await {
            tracing::error!("Discord client error: {:?}", why);
        }
    });

    // Run HTTP server in foreground
    server_future.await;

    // Cleanup
    bot_handle.abort();
    info!("Shutdown complete");
    
    Ok(())
}
