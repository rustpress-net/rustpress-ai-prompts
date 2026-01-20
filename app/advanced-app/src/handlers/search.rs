//! Search Handlers

use crate::models::*;
use crate::services::ServiceError;
use crate::BlogServices;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// GET /search - Search posts
pub async fn search_posts(
    State(services): State<Arc<BlogServices>>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // Validate minimum query length
    if query.q.trim().len() < 3 {
        return Err(ServiceError::Validation(
            "Search query must be at least 3 characters".into(),
        ));
    }

    let results = services.search.search(&query).await?;

    Ok(Json(results))
}
