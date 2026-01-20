//! Custom Axum Extractors
//!
//! Extractors for authentication and request metadata.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use uuid::Uuid;

use crate::auth::AccessTokenClaims;

/// Authenticated user information extracted from JWT claims
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    /// Create user from JWT claims
    pub fn from_claims(claims: &AccessTokenClaims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email.clone(),
            name: claims.name.clone(),
            role: claims.role.clone(),
            email_verified_at: None, // Not included in JWT, fetch from DB if needed
        }
    }

    /// Check if user has admin role
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Check if user can publish content
    pub fn can_publish(&self) -> bool {
        matches!(self.role.as_str(), "author" | "editor" | "admin")
    }

    /// Check if user can moderate content
    pub fn can_moderate(&self) -> bool {
        matches!(self.role.as_str(), "editor" | "admin")
    }
}

/// Extractor for authenticated user
///
/// This extractor retrieves JWT claims from request extensions (set by middleware)
/// or validates the JWT token directly if not present in extensions.
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // First, check if claims were already validated by middleware
        if let Some(claims) = parts.extensions.get::<AccessTokenClaims>() {
            return Ok(AuthUser(User::from_claims(claims)));
        }

        // If not in extensions, validate token directly (for routes without auth middleware)
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

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

        // Get JWT configuration from environment
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

        let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "rustpress".to_string());
        let audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "rustpress-api".to_string());

        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::default();
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);

        let token_data =
            decode::<AccessTokenClaims>(token, &decoding_key, &validation).map_err(|e| {
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

        Ok(AuthUser(User::from_claims(&token_data.claims)))
    }
}

/// Optional authenticated user (doesn't fail if not authenticated)
#[async_trait]
impl<S> FromRequestParts<S> for Option<AuthUser>
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(AuthUser::from_request_parts(parts, state).await.ok())
    }
}

/// Client information (IP, user agent)
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for ClientInfo
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ip = parts
            .headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
            .or_else(|| {
                parts
                    .headers
                    .get("X-Real-IP")
                    .and_then(|h| h.to_str().ok())
                    .map(String::from)
            });

        let user_agent = parts
            .headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(String::from);

        Ok(ClientInfo { ip, user_agent })
    }
}

/// Pagination parameters extractor
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 10,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");

        let mut page = 1i64;
        let mut per_page = 10i64;

        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                match key {
                    "page" => {
                        if let Ok(p) = value.parse() {
                            page = p.max(1);
                        }
                    }
                    "per_page" | "limit" => {
                        if let Ok(pp) = value.parse() {
                            per_page = pp.min(100).max(1);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(Pagination { page, per_page })
    }
}
