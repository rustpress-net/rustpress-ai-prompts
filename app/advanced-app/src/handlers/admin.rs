//! Admin Handlers

use crate::models::*;
use crate::services::ServiceError;
use crate::BlogServices;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// GET /admin/posts - List all posts (admin view)
pub async fn list_all_posts(
    State(services): State<Arc<BlogServices>>,
    Query(query): Query<PostQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // Admin can see all posts regardless of status
    let posts = services.posts.list_published(&query).await?;
    Ok(Json(posts))
}

/// GET /admin/comments/pending - List pending comments
pub async fn pending_comments(
    State(services): State<Arc<BlogServices>>,
) -> Result<impl IntoResponse, ServiceError> {
    // This would be implemented in CommentService
    // For now, return empty list
    Ok(Json(serde_json::json!({
        "data": [],
        "count": 0
    })))
}

/// GET /admin/stats - Blog statistics
pub async fn blog_stats(
    State(services): State<Arc<BlogServices>>,
) -> Result<impl IntoResponse, ServiceError> {
    // This would aggregate stats from the database
    let stats = BlogStats {
        total_posts: 0,
        published_posts: 0,
        draft_posts: 0,
        total_comments: 0,
        pending_comments: 0,
        total_categories: 0,
        total_tags: 0,
        total_media: 0,
        total_views: 0,
    };

    Ok(Json(stats))
}
