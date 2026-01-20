//! Response Caching Middleware

use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::Response,
};

/// Cache GET responses
pub async fn cache_response(req: Request, next: Next) -> Response {
    // Only cache GET requests
    if req.method() != Method::GET {
        return next.run(req).await;
    }

    let path = req.uri().path().to_string();

    // Skip caching for certain paths
    if path.contains("/admin/") || path.contains("/realtime") {
        return next.run(req).await;
    }

    // Check if cached response exists (would use Redis/memory cache)
    // For now, just add cache headers

    let mut response = next.run(req).await;

    // Add cache control headers for public endpoints
    if response.status() == StatusCode::OK {
        let headers = response.headers_mut();

        // Cache for 5 minutes
        headers.insert(
            header::CACHE_CONTROL,
            "public, max-age=300".parse().unwrap(),
        );

        // Add ETag for conditional requests
        // headers.insert(header::ETAG, "\"abc123\"".parse().unwrap());
    }

    response
}
