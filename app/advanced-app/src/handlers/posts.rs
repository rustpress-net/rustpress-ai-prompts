//! Post Handlers

use crate::extractors::AuthUser;
use crate::models::*;
use crate::services::ServiceError;
use crate::BlogServices;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// GET /posts - List published posts
pub async fn list_posts(
    State(services): State<Arc<BlogServices>>,
    Query(query): Query<PostQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let posts = services.posts.list_published(&query).await?;
    Ok(Json(posts))
}

/// GET /posts/:slug - Get post by slug
pub async fn get_post_by_slug(
    State(services): State<Arc<BlogServices>>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, ServiceError> {
    let post = services.posts.get_by_slug(&slug).await?;
    Ok(Json(post))
}

/// POST /posts - Create a new post
pub async fn create_post(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let post = services.posts.create(user.id, req).await?;

    Ok((StatusCode::CREATED, Json(post)))
}

/// PUT /posts/:id - Update a post
pub async fn update_post(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let post = services.posts.update(id, user.id, req).await?;

    Ok(Json(post))
}

/// DELETE /posts/:id - Delete a post
pub async fn delete_post(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    services.posts.delete(id, user.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /posts/:id/publish - Publish a post
pub async fn publish_post(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let post = services.posts.publish(id).await?;

    Ok(Json(post))
}

/// POST /posts/:id/unpublish - Unpublish a post
pub async fn unpublish_post(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let post = services.posts.unpublish(id).await?;

    Ok(Json(post))
}

/// GET /drafts - List user's draft posts
pub async fn list_drafts(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    Query(query): Query<PostQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let mut query = query;
    query.author = Some(user.id);
    query.status = Some(PostStatus::Draft);

    let posts = services.posts.list_published(&query).await?;

    Ok(Json(posts))
}
