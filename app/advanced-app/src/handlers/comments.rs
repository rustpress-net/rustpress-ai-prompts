//! Comment Handlers

use crate::extractors::{AuthUser, ClientInfo};
use crate::models::*;
use crate::services::ServiceError;
use crate::BlogServices;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// GET /posts/:id/comments - List comments for a post
pub async fn list_comments(
    State(services): State<Arc<BlogServices>>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let comments = services.comments.list_for_post(post_id).await?;
    Ok(Json(serde_json::json!({
        "data": comments,
        "count": comments.len()
    })))
}

/// POST /posts/:id/comments - Create a comment
pub async fn create_comment(
    State(services): State<Arc<BlogServices>>,
    Path(post_id): Path<Uuid>,
    auth_user: Option<AuthUser>,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(req): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let author_id = auth_user.map(|a| a.0.id);

    // Comments require moderation unless from authenticated users
    let requires_moderation = author_id.is_none();

    let comment = services
        .comments
        .create(post_id, author_id, req, ip, user_agent, requires_moderation)
        .await?;

    let status = if requires_moderation {
        StatusCode::ACCEPTED
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(comment)))
}

/// POST /comments/:id/approve - Approve a comment
pub async fn approve_comment(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let comment = services.comments.approve(id).await?;
    Ok(Json(comment))
}

/// POST /comments/:id/reject - Reject a comment
pub async fn reject_comment(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let comment = services.comments.reject(id).await?;
    Ok(Json(comment))
}
