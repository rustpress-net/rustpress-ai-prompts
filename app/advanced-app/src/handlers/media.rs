//! Media Handlers

use crate::extractors::AuthUser;
use crate::models::*;
use crate::services::ServiceError;
use crate::BlogServices;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Allowed MIME types for upload
const ALLOWED_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "application/pdf",
];

/// Max file size: 50MB
const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

/// GET /media - List media files
pub async fn list_media(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    Query(query): Query<MediaQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let media = services.media.list(user.id, &query).await?;
    Ok(Json(serde_json::json!({
        "data": media,
        "count": media.len()
    })))
}

/// POST /media - Upload media file
pub async fn upload_media(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ServiceError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServiceError::Validation(e.to_string()))?
    {
        let name = field.name().unwrap_or("file").to_string();

        if name != "file" {
            continue;
        }

        let filename = field
            .file_name()
            .ok_or_else(|| ServiceError::Validation("No filename provided".into()))?
            .to_string();

        let content_type = field
            .content_type()
            .ok_or_else(|| ServiceError::Validation("No content type provided".into()))?
            .to_string();

        // Validate MIME type
        if !ALLOWED_TYPES.contains(&content_type.as_str()) {
            return Err(ServiceError::Validation(format!(
                "File type '{}' not allowed. Allowed types: {:?}",
                content_type, ALLOWED_TYPES
            )));
        }

        // Read file data
        let data = field
            .bytes()
            .await
            .map_err(|e| ServiceError::Validation(e.to_string()))?
            .to_vec();

        // Validate file size
        if data.len() > MAX_FILE_SIZE {
            return Err(ServiceError::Validation(format!(
                "File too large. Max size: {}MB",
                MAX_FILE_SIZE / 1024 / 1024
            )));
        }

        let media = services
            .media
            .upload(user.id, filename, data, content_type)
            .await?;

        return Ok((StatusCode::CREATED, Json(media)));
    }

    Err(ServiceError::Validation("No file uploaded".into()))
}

/// DELETE /media/:id - Delete media file
pub async fn delete_media(
    State(services): State<Arc<BlogServices>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    services.media.delete(id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
