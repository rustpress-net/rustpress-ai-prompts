//! Analytics Services

use crate::models::*;
use crate::AnalyticsConfig;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================
// Tracking Service
// ============================================

pub struct TrackingService {
    db: PgPool,
    config: AnalyticsConfig,
    geoip: Option<maxminddb::Reader<Vec<u8>>>,
}

impl TrackingService {
    pub fn new(db: PgPool, config: AnalyticsConfig) -> Self {
        // Try to load GeoIP database
        let geoip = maxminddb::Reader::open_readfile("data/GeoLite2-City.mmdb").ok();

        Self { db, config, geoip }
    }

    /// Track a page view
    pub async fn track_pageview(
        &self,
        input: &TrackingInput,
        ip: Option<IpAddr>,
        user_agent: &str,
    ) -> Result<(Uuid, Uuid), TrackingError> {
        // Check if tracking is enabled
        if !self.config.tracking_enabled {
            return Err(TrackingError::Disabled);
        }

        // Check excluded paths
        if self.config.excluded_paths.iter().any(|p| input.path.starts_with(p)) {
            return Err(TrackingError::ExcludedPath);
        }

        // Check excluded IPs
        if let Some(ip) = ip {
            let ip_str = ip.to_string();
            if self.config.excluded_ips.contains(&ip_str) {
                return Err(TrackingError::ExcludedIP);
            }
        }

        // Parse user agent
        let ua = user_agent_parser::parse(user_agent);
        let device_type = self.detect_device_type(&ua);
        let browser = ua.browser.map(|b| b.name).unwrap_or("Unknown").to_string();
        let os = ua.os.map(|o| o.name).unwrap_or("Unknown").to_string();

        // Get or create visitor/session
        let visitor_id = input.visitor_id.unwrap_or_else(Uuid::new_v4);
        let session_id = self.get_or_create_session(
            visitor_id,
            &input.path,
            &device_type,
            &browser,
            &os,
            ip,
        ).await?;

        // Anonymize IP if configured
        let stored_ip = if self.config.anonymize_ip {
            ip.map(|i| self.anonymize_ip(i))
        } else {
            ip.map(|i| i.to_string())
        };

        // Get geolocation
        let (country, city) = self.get_geolocation(ip);

        // Insert page view
        sqlx::query!(
            r#"
            INSERT INTO analytics_pageviews
            (session_id, visitor_id, path, title, referrer, utm_source, utm_medium, utm_campaign, ip_address, country, city)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            session_id,
            visitor_id,
            input.path,
            input.title,
            input.referrer,
            input.utm_source,
            input.utm_medium,
            input.utm_campaign,
            stored_ip,
            country,
            city,
        )
        .execute(&self.db)
        .await
        .map_err(|e| TrackingError::Database(e.to_string()))?;

        // Update session
        sqlx::query!(
            r#"
            UPDATE analytics_sessions
            SET page_views = page_views + 1,
                exit_page = $1,
                ended_at = NOW(),
                is_bounce = (page_views = 0)
            WHERE id = $2
            "#,
            input.path,
            session_id,
        )
        .execute(&self.db)
        .await
        .map_err(|e| TrackingError::Database(e.to_string()))?;

        Ok((visitor_id, session_id))
    }

    /// Track a custom event
    pub async fn track_event(
        &self,
        input: &TrackingInput,
    ) -> Result<(), TrackingError> {
        if !self.config.tracking_enabled {
            return Err(TrackingError::Disabled);
        }

        let visitor_id = input.visitor_id.ok_or(TrackingError::MissingVisitorId)?;
        let session_id = input.session_id.ok_or(TrackingError::MissingSessionId)?;

        sqlx::query!(
            r#"
            INSERT INTO analytics_events
            (session_id, visitor_id, category, action, label, value, path)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            session_id,
            visitor_id,
            input.category.as_deref().unwrap_or("general"),
            input.action.as_deref().unwrap_or("click"),
            input.label,
            input.value,
            input.path,
        )
        .execute(&self.db)
        .await
        .map_err(|e| TrackingError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_or_create_session(
        &self,
        visitor_id: Uuid,
        entry_page: &str,
        device_type: &str,
        browser: &str,
        os: &str,
        ip: Option<IpAddr>,
    ) -> Result<Uuid, TrackingError> {
        // Check for existing active session (within last 30 minutes)
        let cutoff = Utc::now() - Duration::minutes(30);

        let existing = sqlx::query_scalar!(
            r#"
            SELECT id FROM analytics_sessions
            WHERE visitor_id = $1 AND ended_at > $2
            ORDER BY ended_at DESC LIMIT 1
            "#,
            visitor_id,
            cutoff,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| TrackingError::Database(e.to_string()))?;

        if let Some(session_id) = existing {
            return Ok(session_id);
        }

        // Create new session
        let session_id = Uuid::new_v4();
        let (country, city) = self.get_geolocation(ip);

        sqlx::query!(
            r#"
            INSERT INTO analytics_sessions
            (id, visitor_id, entry_page, device_type, browser, os, country, city, page_views, is_bounce)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, true)
            "#,
            session_id,
            visitor_id,
            entry_page,
            device_type,
            browser,
            os,
            country,
            city,
        )
        .execute(&self.db)
        .await
        .map_err(|e| TrackingError::Database(e.to_string()))?;

        Ok(session_id)
    }

    fn detect_device_type(&self, ua: &user_agent_parser::UserAgent) -> String {
        if let Some(device) = &ua.device {
            if device.name.to_lowercase().contains("mobile") {
                return "mobile".into();
            }
            if device.name.to_lowercase().contains("tablet") {
                return "tablet".into();
            }
        }
        "desktop".into()
    }

    fn anonymize_ip(&self, ip: IpAddr) -> String {
        match ip {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                format!("{}.{}.{}.0", octets[0], octets[1], octets[2])
            }
            IpAddr::V6(v6) => {
                let segments = v6.segments();
                format!("{:x}:{:x}:{:x}::0", segments[0], segments[1], segments[2])
            }
        }
    }

    fn get_geolocation(&self, ip: Option<IpAddr>) -> (Option<String>, Option<String>) {
        let Some(ip) = ip else {
            return (None, None);
        };

        let Some(reader) = &self.geoip else {
            return (None, None);
        };

        if let Ok(city) = reader.lookup::<maxminddb::geoip2::City>(ip) {
            let country = city.country
                .and_then(|c| c.iso_code)
                .map(String::from);
            let city_name = city.city
                .and_then(|c| c.names)
                .and_then(|n| n.get("en").copied())
                .map(String::from);
            return (country, city_name);
        }

        (None, None)
    }
}

// ============================================
// Analytics Service
// ============================================

pub struct AnalyticsService {
    db: PgPool,
    redis: deadpool_redis::Pool,
}

impl AnalyticsService {
    pub fn new(db: PgPool, redis: deadpool_redis::Pool) -> Self {
        Self { db, redis }
    }

    /// Get real-time active visitors
    pub async fn get_realtime_visitors(&self) -> Result<Vec<RealtimeVisitor>, AnalyticsError> {
        let cutoff = Utc::now() - Duration::minutes(5);

        let visitors = sqlx::query_as!(
            RealtimeVisitor,
            r#"
            SELECT DISTINCT ON (s.visitor_id)
                s.visitor_id,
                p.path as current_page,
                p.title as page_title,
                p.referrer,
                s.device_type,
                s.country,
                s.started_at,
                s.page_views
            FROM analytics_sessions s
            JOIN analytics_pageviews p ON p.session_id = s.id
            WHERE s.ended_at > $1
            ORDER BY s.visitor_id, p.created_at DESC
            "#,
            cutoff,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(visitors)
    }

    /// Get page views for a period
    pub async fn get_pageviews(&self, query: &ReportQuery) -> Result<Vec<PageView>, AnalyticsError> {
        let (from, to) = query.date_range();
        let limit = query.limit.unwrap_or(100).min(1000);
        let offset = query.offset.unwrap_or(0);

        let views = sqlx::query_as!(
            PageView,
            r#"
            SELECT id, session_id, visitor_id, path, title, referrer,
                   utm_source, utm_medium, utm_campaign, created_at
            FROM analytics_pageviews
            WHERE created_at::date BETWEEN $1 AND $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            from,
            to,
            limit,
            offset,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(views)
    }

    /// Get daily statistics
    pub async fn get_daily_stats(&self, query: &ReportQuery) -> Result<Vec<DailyStats>, AnalyticsError> {
        let (from, to) = query.date_range();

        let stats = sqlx::query_as!(
            DailyStats,
            r#"
            SELECT date, page_views, unique_visitors, sessions,
                   bounce_rate, avg_session_duration, new_visitors, returning_visitors
            FROM analytics_daily_stats
            WHERE date BETWEEN $1 AND $2
            ORDER BY date ASC
            "#,
            from,
            to,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(stats)
    }
}

// ============================================
// Report Service
// ============================================

pub struct ReportService {
    db: PgPool,
}

impl ReportService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Generate overview report
    pub async fn get_overview(&self, query: &ReportQuery) -> Result<OverviewReport, ReportError> {
        let (from, to) = query.date_range();

        // Get totals
        let totals = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(page_views), 0) as total_page_views,
                COALESCE(SUM(unique_visitors), 0) as unique_visitors,
                COALESCE(SUM(sessions), 0) as total_sessions,
                COALESCE(AVG(bounce_rate), 0) as bounce_rate,
                COALESCE(AVG(avg_session_duration), 0) as avg_session_duration,
                COALESCE(SUM(new_visitors), 0) as new_visitors,
                COALESCE(SUM(returning_visitors), 0) as returning_visitors
            FROM analytics_daily_stats
            WHERE date BETWEEN $1 AND $2
            "#,
            from,
            to,
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| ReportError::Database(e.to_string()))?;

        // Get daily breakdown
        let daily_stats = sqlx::query_as!(
            DailyStats,
            r#"
            SELECT date, page_views, unique_visitors, sessions,
                   bounce_rate, avg_session_duration, new_visitors, returning_visitors
            FROM analytics_daily_stats
            WHERE date BETWEEN $1 AND $2
            ORDER BY date ASC
            "#,
            from,
            to,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ReportError::Database(e.to_string()))?;

        let total_visitors = totals.new_visitors.unwrap_or(0) + totals.returning_visitors.unwrap_or(0);
        let new_percentage = if total_visitors > 0 {
            (totals.new_visitors.unwrap_or(0) as f64 / total_visitors as f64) * 100.0
        } else {
            0.0
        };

        let sessions = totals.total_sessions.unwrap_or(0);
        let pages_per_session = if sessions > 0 {
            totals.total_page_views.unwrap_or(0) as f64 / sessions as f64
        } else {
            0.0
        };

        Ok(OverviewReport {
            period: query.period.clone().unwrap_or_else(|| "30d".into()),
            total_page_views: totals.total_page_views.unwrap_or(0),
            unique_visitors: totals.unique_visitors.unwrap_or(0),
            total_sessions: sessions,
            bounce_rate: totals.bounce_rate.unwrap_or(0.0),
            avg_session_duration: totals.avg_session_duration.unwrap_or(0.0),
            pages_per_session,
            new_vs_returning: NewVsReturning {
                new_visitors: totals.new_visitors.unwrap_or(0),
                returning_visitors: totals.returning_visitors.unwrap_or(0),
                new_percentage,
            },
            daily_stats,
        })
    }

    /// Get top pages report
    pub async fn get_pages(&self, query: &ReportQuery) -> Result<Vec<PageReport>, ReportError> {
        let (from, to) = query.date_range();
        let limit = query.limit.unwrap_or(20);

        let pages = sqlx::query_as!(
            PageReport,
            r#"
            SELECT
                p.path,
                MAX(p.title) as title,
                COUNT(*) as page_views,
                COUNT(DISTINCT p.visitor_id) as unique_visitors,
                AVG(EXTRACT(EPOCH FROM (LEAD(p.created_at) OVER (PARTITION BY p.session_id ORDER BY p.created_at) - p.created_at))) as avg_time_on_page,
                (COUNT(*) FILTER (WHERE s.is_bounce AND s.entry_page = p.path)::float / NULLIF(COUNT(*), 0)) * 100 as bounce_rate,
                COUNT(*) FILTER (WHERE s.entry_page = p.path) as entrances,
                COUNT(*) FILTER (WHERE s.exit_page = p.path) as exits
            FROM analytics_pageviews p
            JOIN analytics_sessions s ON s.id = p.session_id
            WHERE p.created_at::date BETWEEN $1 AND $2
            GROUP BY p.path
            ORDER BY page_views DESC
            LIMIT $3
            "#,
            from,
            to,
            limit,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ReportError::Database(e.to_string()))?;

        Ok(pages)
    }

    /// Get referrers report
    pub async fn get_referrers(&self, query: &ReportQuery) -> Result<Vec<ReferrerReport>, ReportError> {
        let (from, to) = query.date_range();
        let limit = query.limit.unwrap_or(20);

        let referrers = sqlx::query_as!(
            ReferrerReport,
            r#"
            SELECT
                COALESCE(p.referrer, 'Direct') as referrer,
                COUNT(DISTINCT p.session_id) as sessions,
                COUNT(*) as page_views,
                (COUNT(*) FILTER (WHERE s.is_bounce)::float / NULLIF(COUNT(DISTINCT p.session_id), 0)) * 100 as bounce_rate,
                AVG(s.duration_seconds) as avg_session_duration
            FROM analytics_pageviews p
            JOIN analytics_sessions s ON s.id = p.session_id
            WHERE p.created_at::date BETWEEN $1 AND $2
            GROUP BY COALESCE(p.referrer, 'Direct')
            ORDER BY sessions DESC
            LIMIT $3
            "#,
            from,
            to,
            limit,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ReportError::Database(e.to_string()))?;

        Ok(referrers)
    }

    /// Get device breakdown
    pub async fn get_devices(&self, query: &ReportQuery) -> Result<Vec<DeviceReport>, ReportError> {
        let (from, to) = query.date_range();

        let devices = sqlx::query_as!(
            DeviceReport,
            r#"
            SELECT
                device_type,
                COUNT(*) as sessions,
                (COUNT(*)::float / SUM(COUNT(*)) OVER ()) * 100 as percentage
            FROM analytics_sessions
            WHERE started_at::date BETWEEN $1 AND $2
            GROUP BY device_type
            ORDER BY sessions DESC
            "#,
            from,
            to,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ReportError::Database(e.to_string()))?;

        Ok(devices)
    }

    /// Get geography report
    pub async fn get_geography(&self, query: &ReportQuery) -> Result<Vec<GeoReport>, ReportError> {
        let (from, to) = query.date_range();
        let limit = query.limit.unwrap_or(20);

        let geo = sqlx::query_as!(
            GeoReport,
            r#"
            SELECT
                COALESCE(country, 'Unknown') as country,
                COUNT(*) as sessions,
                SUM(page_views) as page_views,
                (COUNT(*)::float / SUM(COUNT(*)) OVER ()) * 100 as percentage
            FROM analytics_sessions
            WHERE started_at::date BETWEEN $1 AND $2
            GROUP BY country
            ORDER BY sessions DESC
            LIMIT $3
            "#,
            from,
            to,
            limit,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ReportError::Database(e.to_string()))?;

        Ok(geo)
    }
}

// ============================================
// Error Types
// ============================================

#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    #[error("Tracking is disabled")]
    Disabled,
    #[error("Path is excluded")]
    ExcludedPath,
    #[error("IP is excluded")]
    ExcludedIP,
    #[error("Missing visitor ID")]
    MissingVisitorId,
    #[error("Missing session ID")]
    MissingSessionId,
    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Export error: {0}")]
    Export(String),
}
