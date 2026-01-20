//! Analytics Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A tracked page view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PageView {
    pub id: i64,
    pub session_id: Uuid,
    pub visitor_id: Uuid,
    pub path: String,
    pub title: Option<String>,
    pub referrer: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A visitor session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub visitor_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub page_views: i32,
    pub duration_seconds: Option<i32>,
    pub entry_page: String,
    pub exit_page: Option<String>,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub is_bounce: bool,
}

/// A tracked event (clicks, downloads, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i64,
    pub session_id: Uuid,
    pub visitor_id: Uuid,
    pub category: String,
    pub action: String,
    pub label: Option<String>,
    pub value: Option<i32>,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

/// Daily aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub page_views: i64,
    pub unique_visitors: i64,
    pub sessions: i64,
    pub bounce_rate: f64,
    pub avg_session_duration: f64,
    pub new_visitors: i64,
    pub returning_visitors: i64,
}

/// Real-time visitor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeVisitor {
    pub visitor_id: Uuid,
    pub current_page: String,
    pub page_title: Option<String>,
    pub referrer: Option<String>,
    pub device_type: String,
    pub country: Option<String>,
    pub started_at: DateTime<Utc>,
    pub page_views: i32,
}

/// Report data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewReport {
    pub period: String,
    pub total_page_views: i64,
    pub unique_visitors: i64,
    pub total_sessions: i64,
    pub bounce_rate: f64,
    pub avg_session_duration: f64,
    pub pages_per_session: f64,
    pub new_vs_returning: NewVsReturning,
    pub daily_stats: Vec<DailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVsReturning {
    pub new_visitors: i64,
    pub returning_visitors: i64,
    pub new_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageReport {
    pub path: String,
    pub title: Option<String>,
    pub page_views: i64,
    pub unique_visitors: i64,
    pub avg_time_on_page: f64,
    pub bounce_rate: f64,
    pub entrances: i64,
    pub exits: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerReport {
    pub referrer: String,
    pub sessions: i64,
    pub page_views: i64,
    pub bounce_rate: f64,
    pub avg_session_duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceReport {
    pub device_type: String,
    pub sessions: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserReport {
    pub browser: String,
    pub sessions: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoReport {
    pub country: String,
    pub sessions: i64,
    pub page_views: i64,
    pub percentage: f64,
}

/// Input for tracking events
#[derive(Debug, Clone, Deserialize)]
pub struct TrackingInput {
    pub visitor_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub event_type: String, // "pageview" | "event"
    pub path: String,
    pub title: Option<String>,
    pub referrer: Option<String>,
    pub category: Option<String>,
    pub action: Option<String>,
    pub label: Option<String>,
    pub value: Option<i32>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
}

/// Query parameters for reports
#[derive(Debug, Clone, Deserialize)]
pub struct ReportQuery {
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub period: Option<String>, // "7d", "30d", "90d", "365d", "custom"
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ReportQuery {
    pub fn date_range(&self) -> (chrono::NaiveDate, chrono::NaiveDate) {
        let today = Utc::now().date_naive();

        if let (Some(from), Some(to)) = (self.from, self.to) {
            return (from, to);
        }

        let days = match self.period.as_deref() {
            Some("7d") => 7,
            Some("90d") => 90,
            Some("365d") => 365,
            _ => 30,
        };

        (today - chrono::Duration::days(days), today)
    }
}
