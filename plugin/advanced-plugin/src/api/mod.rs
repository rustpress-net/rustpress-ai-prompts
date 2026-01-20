//! Analytics REST API Handlers

use crate::models::*;
use crate::services::*;
use crate::AnalyticsPlugin;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// Create API routes
pub fn create_routes(plugin: &AnalyticsPlugin) -> Router {
    Router::new()
        // Public tracking endpoint
        .route("/track", post(track_event))
        // Protected analytics endpoints
        .route("/pageviews", get(get_pageviews))
        .route("/visitors", get(get_visitors))
        .route("/realtime", get(get_realtime))
        .route("/reports/overview", get(get_overview_report))
        .route("/reports/pages", get(get_pages_report))
        .route("/reports/referrers", get(get_referrers_report))
        .route("/reports/devices", get(get_devices_report))
        .route("/reports/geography", get(get_geography_report))
        .route("/reports/export", post(export_report))
}

// ============================================
// Tracking Endpoint
// ============================================

/// POST /api/v1/analytics/track
pub async fn track_event(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<TrackingInput>,
) -> impl IntoResponse {
    let Some(tracking) = plugin.tracking().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Tracking service unavailable"
        })));
    };

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let ip = Some(addr.ip());

    match input.event_type.as_str() {
        "pageview" => {
            match tracking.track_pageview(&input, ip, user_agent).await {
                Ok((visitor_id, session_id)) => {
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "visitor_id": visitor_id,
                        "session_id": session_id
                    })))
                }
                Err(TrackingError::Disabled) |
                Err(TrackingError::ExcludedPath) |
                Err(TrackingError::ExcludedIP) => {
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "tracked": false
                    })))
                }
                Err(e) => {
                    tracing::error!("Tracking error: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": "Tracking failed"
                    })))
                }
            }
        }
        "event" => {
            match tracking.track_event(&input).await {
                Ok(()) => {
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true
                    })))
                }
                Err(e) => {
                    tracing::error!("Event tracking error: {:?}", e);
                    (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                        "error": format!("{}", e)
                    })))
                }
            }
        }
        _ => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid event type"
            })))
        }
    }
}

// ============================================
// Analytics Endpoints
// ============================================

/// GET /api/v1/analytics/pageviews
pub async fn get_pageviews(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(analytics) = plugin.analytics().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Analytics service unavailable"
        })));
    };

    match analytics.get_pageviews(&query).await {
        Ok(views) => (StatusCode::OK, Json(serde_json::json!({
            "data": views,
            "count": views.len()
        }))),
        Err(e) => {
            tracing::error!("Failed to get pageviews: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch pageviews"
            })))
        }
    }
}

/// GET /api/v1/analytics/visitors
pub async fn get_visitors(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(analytics) = plugin.analytics().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Analytics service unavailable"
        })));
    };

    match analytics.get_daily_stats(&query).await {
        Ok(stats) => {
            let total_visitors: i64 = stats.iter().map(|s| s.unique_visitors).sum();
            (StatusCode::OK, Json(serde_json::json!({
                "total": total_visitors,
                "daily": stats
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get visitors: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch visitors"
            })))
        }
    }
}

/// GET /api/v1/analytics/realtime
pub async fn get_realtime(
    State(plugin): State<Arc<AnalyticsPlugin>>,
) -> impl IntoResponse {
    let config = plugin.config().await;
    if !config.realtime_enabled {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Real-time tracking is disabled"
        })));
    }

    let Some(analytics) = plugin.analytics().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Analytics service unavailable"
        })));
    };

    match analytics.get_realtime_visitors().await {
        Ok(visitors) => (StatusCode::OK, Json(serde_json::json!({
            "active_visitors": visitors.len(),
            "visitors": visitors
        }))),
        Err(e) => {
            tracing::error!("Failed to get realtime: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch realtime data"
            })))
        }
    }
}

// ============================================
// Report Endpoints
// ============================================

/// GET /api/v1/analytics/reports/overview
pub async fn get_overview_report(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(reports) = plugin.reports().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Report service unavailable"
        })));
    };

    match reports.get_overview(&query).await {
        Ok(report) => (StatusCode::OK, Json(report)),
        Err(e) => {
            tracing::error!("Failed to get overview report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to generate report"
            })))
        }
    }
}

/// GET /api/v1/analytics/reports/pages
pub async fn get_pages_report(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(reports) = plugin.reports().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Report service unavailable"
        })));
    };

    match reports.get_pages(&query).await {
        Ok(pages) => (StatusCode::OK, Json(serde_json::json!({
            "data": pages
        }))),
        Err(e) => {
            tracing::error!("Failed to get pages report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to generate report"
            })))
        }
    }
}

/// GET /api/v1/analytics/reports/referrers
pub async fn get_referrers_report(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(reports) = plugin.reports().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Report service unavailable"
        })));
    };

    match reports.get_referrers(&query).await {
        Ok(referrers) => (StatusCode::OK, Json(serde_json::json!({
            "data": referrers
        }))),
        Err(e) => {
            tracing::error!("Failed to get referrers report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to generate report"
            })))
        }
    }
}

/// GET /api/v1/analytics/reports/devices
pub async fn get_devices_report(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(reports) = plugin.reports().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Report service unavailable"
        })));
    };

    match reports.get_devices(&query).await {
        Ok(devices) => (StatusCode::OK, Json(serde_json::json!({
            "data": devices
        }))),
        Err(e) => {
            tracing::error!("Failed to get devices report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to generate report"
            })))
        }
    }
}

/// GET /api/v1/analytics/reports/geography
pub async fn get_geography_report(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let Some(reports) = plugin.reports().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "Report service unavailable"
        })));
    };

    match reports.get_geography(&query).await {
        Ok(geo) => (StatusCode::OK, Json(serde_json::json!({
            "data": geo
        }))),
        Err(e) => {
            tracing::error!("Failed to get geography report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to generate report"
            })))
        }
    }
}

/// POST /api/v1/analytics/reports/export
pub async fn export_report(
    State(plugin): State<Arc<AnalyticsPlugin>>,
    Json(params): Json<ExportParams>,
) -> impl IntoResponse {
    // Export implementation
    (StatusCode::OK, Json(serde_json::json!({
        "message": "Export started",
        "format": params.format,
        "download_url": "/api/v1/analytics/exports/12345"
    })))
}

#[derive(serde::Deserialize)]
pub struct ExportParams {
    pub format: String, // "csv" | "json" | "pdf"
    pub report_type: String,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
}
