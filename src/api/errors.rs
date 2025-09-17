use serde::Serialize;
use std::fmt;
use warp::{Rejection, Reply};
use warp::http::StatusCode;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    DatabaseError(String),
    CacheError(String),
    NotFound,
    InvalidInput(String),
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ApiError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            ApiError::NotFound => write!(f, "Resource not found"),
            ApiError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl warp::reject::Reject for ApiError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

/// Convert API errors to HTTP responses
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let (code, message) = if let Some(api_error) = err.find::<ApiError>() {
        match api_error {
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::CacheError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        }
    } else if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Route not found".to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
    };

    let json = warp::reply::json(&ErrorResponse {
        error: message,
        code: code.as_u16(),
    });

    Ok(warp::reply::with_status(json, code))
}
