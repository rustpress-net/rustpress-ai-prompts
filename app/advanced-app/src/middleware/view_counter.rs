//! View Counter Middleware

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// Increment view count for posts
pub async fn increment_views(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();

    // Process the request first
    let response = next.run(req).await;

    // Only count successful requests
    if response.status().is_success() && path.starts_with("/posts/") {
        // Extract slug from path and increment view count
        // This would be done asynchronously to not block the response
        let slug = path.trim_start_matches("/posts/");
        tracing::debug!("Incrementing view count for post: {}", slug);

        // Would call: services.posts.increment_views(post_id).await
    }

    response
}
