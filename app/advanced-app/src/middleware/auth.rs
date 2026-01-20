//! Authentication Middleware

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use rustpress_apps::prelude::*;

/// Require authenticated user
pub async fn require_auth(req: Request, next: Next) -> Result<Response, Response> {
    // Check for valid session/JWT token
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            // Validate token (would use actual JWT validation)
            // For now, pass through
            Ok(next.run(req).await)
        }
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })),
        )
            .into_response()),
    }
}

/// Require admin role
pub async fn require_admin(req: Request, next: Next) -> Result<Response, Response> {
    // First check authentication
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            // Check if user has admin role (would decode JWT and check claims)
            // For now, pass through
            Ok(next.run(req).await)
        }
        _ => Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response()),
    }
}
