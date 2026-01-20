//! Rate Limiting Middleware

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Simple in-memory rate limiter state
struct RateLimitState {
    requests: HashMap<String, (u32, Instant)>,
}

lazy_static::lazy_static! {
    static ref RATE_LIMIT_STATE: Arc<RwLock<RateLimitState>> = Arc::new(RwLock::new(RateLimitState {
        requests: HashMap::new(),
    }));
}

/// Rate limit configuration
const REQUESTS_PER_WINDOW: u32 = 100;
const WINDOW_DURATION: Duration = Duration::from_secs(60);

/// Rate limiting middleware
pub async fn rate_limiter(req: Request, next: Next) -> Result<Response, Response> {
    // Get client identifier (IP address or API key)
    let client_id = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limit
    let mut state = RATE_LIMIT_STATE.write().await;
    let now = Instant::now();

    let (count, window_start) = state
        .requests
        .entry(client_id.clone())
        .or_insert((0, now));

    // Reset if window expired
    if now.duration_since(*window_start) > WINDOW_DURATION {
        *count = 0;
        *window_start = now;
    }

    // Check if over limit
    if *count >= REQUESTS_PER_WINDOW {
        let retry_after = WINDOW_DURATION
            .checked_sub(now.duration_since(*window_start))
            .unwrap_or(Duration::ZERO)
            .as_secs();

        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": "Too many requests. Please try again later.",
                "retry_after": retry_after
            })),
        )
            .into_response());
    }

    // Increment counter
    *count += 1;
    let remaining = REQUESTS_PER_WINDOW - *count;

    drop(state);

    // Process request
    let mut response = next.run(req).await;

    // Add rate limit headers
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", REQUESTS_PER_WINDOW.into());
    headers.insert("X-RateLimit-Remaining", remaining.into());

    Ok(response)
}
