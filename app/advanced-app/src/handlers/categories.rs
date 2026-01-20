//! Category Handlers

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

/// GET /categories - List all categories
pub async fn list_categories(
    State(services): State<Arc<BlogServices>>,
) -> Result<impl IntoResponse, ServiceError> {
    let categories = services.categories.list().await?;
    Ok(Json(serde_json::json!({
        "data": categories
    })))
}

/// POST /categories - Create a category
pub async fn create_category(
    State(services): State<Arc<BlogServices>>,
    Json(req): Json<CategoryRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let category = services.categories.create(req).await?;
    Ok((StatusCode::CREATED, Json(category)))
}

/// PUT /categories/:id - Update a category
pub async fn update_category(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CategoryRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.validate()
        .map_err(|e| ServiceError::Validation(e.to_string()))?;

    let category = services.categories.update(id, req).await?;
    Ok(Json(category))
}

/// DELETE /categories/:id - Delete a category
pub async fn delete_category(
    State(services): State<Arc<BlogServices>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    services.categories.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
