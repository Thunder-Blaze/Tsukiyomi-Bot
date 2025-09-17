use warp::{Filter, Reply, Rejection};
use crate::state::AppState;

/// Create all API routes
pub fn create_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let health_route = warp::path::end()
        .and(warp::method())
        .and_then(|method: warp::http::Method| async move {
            if method == warp::http::Method::HEAD || method == warp::http::Method::GET {
                Ok::<_, warp::Rejection>(warp::reply::with_status("OK", warp::http::StatusCode::OK))
            } else {
                Err(warp::reject::not_found())
            }
        });

    // Dedicated health check endpoint
    let health_check_route = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::with_status("Healthy", warp::http::StatusCode::OK));

    // Get all presences from database
    let app_state_clone1 = app_state.clone();
    let all_presences_route = warp::path!("presences")
        .and(warp::get())
        .and_then(move || {
            let app_state = app_state_clone1.clone();
            async move {
                match app_state.db.get_all_presences(None, None).await {
                    Ok(presences) => {
                        let data: Vec<_> = presences
                            .iter()
                            .map(|p| serde_json::json!({
                                "user_id": p.user_id.to_string(),
                                "status": p.status,
                                "last_seen_at": p.last_seen_at,
                                "updated_at": p.updated_at
                            }))
                            .collect();
                        println!("[HTTP GET /presences] returning {} entries from database", data.len());
                        Ok::<_, warp::Rejection>(warp::reply::json(&data))
                    }
                    Err(e) => {
                        eprintln!("Database error: {}", e);
                        Err(warp::reject::not_found())
                    }
                }
            }
        });

    // Get specific user presence
    let app_state_clone2 = app_state.clone();
    let presence_by_id_route = warp::path!("presences" / u64)
        .and(warp::get())
        .and_then(move |user_id: u64| {
            let app_state = app_state_clone2.clone();
            async move {
                // Try Redis cache first, then database
                match app_state.redis.get_cached_presence(user_id).await {
                    Ok(Some(presence)) => {
                        println!("[HTTP GET /presences/{}] found in cache", user_id);
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            format!("{:?}", presence.status), 
                            warp::http::StatusCode::OK
                        ))
                    }
                    _ => {
                        // Try database
                        match app_state.db.get_user_presence(user_id).await {
                            Ok(Some(presence)) => {
                                println!("[HTTP GET /presences/{}] found in database", user_id);
                                // Cache for next time
                                let _ = app_state.redis.cache_presence(user_id, &presence).await;
                                Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    format!("{:?}", presence.status), 
                                    warp::http::StatusCode::OK
                                ))
                            }
                            _ => {
                                println!("[HTTP GET /presences/{}] not found", user_id);
                                Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    "NotFound".to_string(), 
                                    warp::http::StatusCode::NOT_FOUND
                                ))
                            }
                        }
                    }
                }
            }
        });

    health_route
        .or(health_check_route)
        .or(all_presences_route)
        .or(presence_by_id_route)
}

