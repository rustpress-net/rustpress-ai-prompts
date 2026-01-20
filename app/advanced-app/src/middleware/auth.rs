//! Authentication Middleware
//!
//! JWT token validation middleware using real cryptographic verification.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

use crate::auth::AccessTokenClaims;

/// Get JWT decoding key from environment
fn get_decoding_key() -> Result<DecodingKey, Response> {
    let secret = env::var("JWT_SECRET").map_err(|_| {
        tracing::error!("JWT_SECRET environment variable not set");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "configuration_error",
                "message": "Server configuration error"
            })),
        )
            .into_response()
    })?;
    Ok(DecodingKey::from_secret(secret.as_bytes()))
}

/// Get JWT validation settings from environment
fn get_validation() -> Validation {
    let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "rustpress".to_string());
    let audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "rustpress-api".to_string());

    let mut validation = Validation::default();
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation
}

/// Extract and validate JWT token from Authorization header
fn validate_token(auth_header: Option<&str>) -> Result<AccessTokenClaims, Response> {
    let header = auth_header.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })),
        )
            .into_response()
    })?;

    if !header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Invalid authorization header format"
            })),
        )
            .into_response());
    }

    let token = header.trim_start_matches("Bearer ");
    let decoding_key = get_decoding_key()?;
    let validation = get_validation();

    let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation).map_err(|e| {
        tracing::debug!("JWT validation failed: {:?}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Invalid or expired token"
            })),
        )
            .into_response()
    })?;

    Ok(token_data.claims)
}

/// Require authenticated user
///
/// Validates the JWT token from the Authorization header and stores
/// the claims in request extensions for use by extractors.
pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, Response> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let claims = validate_token(auth_header)?;

    // Store claims in request extensions for extractors
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Require admin role
///
/// Validates JWT and checks that the user has admin role.
pub async fn require_admin(mut req: Request, next: Next) -> Result<Response, Response> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let claims = validate_token(auth_header)?;

    // Check admin role from JWT claims
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response());
    }

    // Store claims in request extensions for extractors
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
