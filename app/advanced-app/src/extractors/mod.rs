//! Custom Axum Extractors

use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use uuid::Uuid;

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
}

/// Extractor for authenticated user
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = header.trim_start_matches("Bearer ");

                // In production, this would validate the JWT and extract claims
                // For this example, we create a mock user
                let user = User {
                    id: Uuid::new_v4(),
                    email: "user@example.com".into(),
                    name: "Example User".into(),
                    is_admin: false,
                };

                Ok(AuthUser(user))
            }
            _ => Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Valid authentication token required"
                })),
            )
                .into_response()),
        }
    }
}

/// Optional authenticated user (doesn't fail if not authenticated)
impl<S> FromRequestParts<S> for Option<AuthUser>
where
    S: Send + Sync,
{
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
