//! Blog API Handlers

pub mod admin;
pub mod categories;
pub mod comments;
pub mod feed;
pub mod media;
pub mod posts;
pub mod search;
pub mod tags;

use crate::models::ApiError;
use crate::services::ServiceError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

/// Convert service errors to HTTP responses
impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            ServiceError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                "permission_denied",
                "You don't have permission to perform this action".to_string(),
            ),
            ServiceError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred".to_string(),
                )
            }
            ServiceError::Storage(msg) => {
                tracing::error!("Storage error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    "A storage error occurred".to_string(),
                )
            }
        };

        (status, Json(ApiError::new(error, &message))).into_response()
    }
}
