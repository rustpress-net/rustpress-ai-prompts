//! Tag Handlers

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

/// GET /tags - List all tags
pub async fn list_tags(
    State(services): State<Arc<BlogServices>>,
) -> Result<impl IntoResponse, ServiceError> {
    let tags = services.tags.list().await?;
    Ok(Json(serde_json::json!({
        "data": tags
    })))
}

/// POST /tags - Create a tag
pub async fn create_tag(
    State(services): State<Arc<BlogServices>>,
    Json(req): Json<TagRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let tag = services.tags.create(req).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

/// PUT /tags/:id - Update a tag
pub async fn update_tag(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
    Json(req): Json<TagRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let tag = services.tags.update(id, req).await?;
    Ok(Json(tag))
}

/// DELETE /tags/:id - Delete a tag
pub async fn delete_tag(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    services.tags.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
