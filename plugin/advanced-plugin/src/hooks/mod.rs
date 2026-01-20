//! Analytics Hook Handlers

use crate::AnalyticsPlugin;
use rustpress_plugins::prelude::*;
use std::sync::Arc;

/// Track page view action
pub async fn track_page_view(
    ctx: ActionContext,
    plugin: Arc<AnalyticsPlugin>,
    data: Box<dyn std::any::Any + Send>,
) -> Result<(), HookError> {
    let config = plugin.config().await;

    if !config.tracking_enabled {
        return Ok(());
    }

    // Check if we should track admin users
    if let Some(user) = &ctx.user {
        if user.is_admin() && !config.track_admins {
            return Ok(());
        }
    }

    // The actual tracking is done via the JS tracker and API
    // This hook is for server-side tracking if needed
    tracing::debug!("Page view hook triggered for request {}", ctx.request_id);

    Ok(())
}

/// Track user login action
pub async fn track_user_login(
    ctx: ActionContext,
    plugin: Arc<AnalyticsPlugin>,
    data: Box<dyn std::any::Any + Send>,
) -> Result<(), HookError> {
    let config = plugin.config().await;

    if !config.tracking_enabled {
        return Ok(());
    }

    if let Some(user_id) = data.downcast_ref::<i64>() {
        tracing::info!(
            request_id = %ctx.request_id,
            user_id = %user_id,
            "User login tracked"
        );

        // Track as event
        if let Some(tracking) = plugin.tracking().await {
            let input = crate::models::TrackingInput {
                visitor_id: None,
                session_id: None,
                event_type: "event".into(),
                path: "/login".into(),
                title: None,
                referrer: None,
                category: Some("user".into()),
                action: Some("login".into()),
                label: Some(format!("user:{}", user_id)),
                value: None,
                utm_source: None,
                utm_medium: None,
                utm_campaign: None,
            };

            if let Err(e) = tracking.track_event(&input).await {
                tracing::warn!("Failed to track login event: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Track content publish action
pub async fn track_content_publish(
    ctx: ActionContext,
    plugin: Arc<AnalyticsPlugin>,
    data: Box<dyn std::any::Any + Send>,
) -> Result<(), HookError> {
    let config = plugin.config().await;

    if !config.tracking_enabled {
        return Ok(());
    }

    if let Some(post_id) = data.downcast_ref::<i64>() {
        tracing::info!(
            request_id = %ctx.request_id,
            post_id = %post_id,
            "Content publish tracked"
        );
    }

    Ok(())
}

/// Inject tracking script into page footer
pub async fn inject_tracking_script(
    ctx: FilterContext,
    plugin: Arc<AnalyticsPlugin>,
    content: String,
) -> Result<String, HookError> {
    let config = plugin.config().await;

    if !config.tracking_enabled {
        return Ok(content);
    }

    // Check if we should track admin users
    if let Some(user) = &ctx.user {
        if user.is_admin() && !config.track_admins {
            return Ok(content);
        }
    }

    let script = format!(
        r#"
<script>
(function() {{
    var analytics = {{
        endpoint: '/api/v1/analytics/track',
        visitorId: localStorage.getItem('_rp_vid') || null,
        sessionId: sessionStorage.getItem('_rp_sid') || null,
        trackOutbound: {},
        trackDownloads: {},
        downloadExtensions: {:?},

        init: function() {{
            this.trackPageView();
            if (this.trackOutbound) this.setupOutboundTracking();
            if (this.trackDownloads) this.setupDownloadTracking();
        }},

        track: function(data) {{
            data.visitor_id = this.visitorId;
            data.session_id = this.sessionId;

            fetch(this.endpoint, {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify(data),
                keepalive: true
            }}).then(function(r) {{ return r.json(); }}).then(function(d) {{
                if (d.visitor_id) {{
                    localStorage.setItem('_rp_vid', d.visitor_id);
                    analytics.visitorId = d.visitor_id;
                }}
                if (d.session_id) {{
                    sessionStorage.setItem('_rp_sid', d.session_id);
                    analytics.sessionId = d.session_id;
                }}
            }});
        }},

        trackPageView: function() {{
            this.track({{
                event_type: 'pageview',
                path: location.pathname,
                title: document.title,
                referrer: document.referrer,
                utm_source: this.getParam('utm_source'),
                utm_medium: this.getParam('utm_medium'),
                utm_campaign: this.getParam('utm_campaign')
            }});
        }},

        trackEvent: function(category, action, label, value) {{
            this.track({{
                event_type: 'event',
                path: location.pathname,
                category: category,
                action: action,
                label: label,
                value: value
            }});
        }},

        setupOutboundTracking: function() {{
            document.addEventListener('click', function(e) {{
                var link = e.target.closest('a');
                if (link && link.hostname !== location.hostname) {{
                    analytics.trackEvent('outbound', 'click', link.href);
                }}
            }});
        }},

        setupDownloadTracking: function() {{
            var exts = this.downloadExtensions;
            document.addEventListener('click', function(e) {{
                var link = e.target.closest('a');
                if (link) {{
                    var ext = link.pathname.split('.').pop().toLowerCase();
                    if (exts.indexOf(ext) > -1) {{
                        analytics.trackEvent('download', ext, link.pathname);
                    }}
                }}
            }});
        }},

        getParam: function(name) {{
            var params = new URLSearchParams(location.search);
            return params.get(name);
        }}
    }};

    analytics.init();
    window.rpAnalytics = analytics;
}})();
</script>
"#,
        config.track_outbound_links,
        config.track_downloads,
        config.download_extensions,
    );

    Ok(format!("{}{}", content, script))
}

/// Cron job: Aggregate daily statistics
pub async fn aggregate_daily_stats(
    ctx: CronContext,
    plugin: Arc<AnalyticsPlugin>,
) -> Result<(), HookError> {
    tracing::info!("Running daily stats aggregation");

    let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);

    sqlx::query!(
        r#"
        INSERT INTO analytics_daily_stats (date, page_views, unique_visitors, sessions, bounce_rate, avg_session_duration, new_visitors, returning_visitors)
        SELECT
            $1::date as date,
            COUNT(p.id) as page_views,
            COUNT(DISTINCT p.visitor_id) as unique_visitors,
            COUNT(DISTINCT p.session_id) as sessions,
            (COUNT(*) FILTER (WHERE s.is_bounce)::float / NULLIF(COUNT(DISTINCT s.id), 0)) * 100,
            AVG(s.duration_seconds),
            COUNT(DISTINCT p.visitor_id) FILTER (WHERE NOT EXISTS (
                SELECT 1 FROM analytics_pageviews p2
                WHERE p2.visitor_id = p.visitor_id AND p2.created_at < $1::date
            )),
            COUNT(DISTINCT p.visitor_id) FILTER (WHERE EXISTS (
                SELECT 1 FROM analytics_pageviews p2
                WHERE p2.visitor_id = p.visitor_id AND p2.created_at < $1::date
            ))
        FROM analytics_pageviews p
        JOIN analytics_sessions s ON s.id = p.session_id
        WHERE p.created_at::date = $1
        ON CONFLICT (date) DO UPDATE SET
            page_views = EXCLUDED.page_views,
            unique_visitors = EXCLUDED.unique_visitors,
            sessions = EXCLUDED.sessions,
            bounce_rate = EXCLUDED.bounce_rate,
            avg_session_duration = EXCLUDED.avg_session_duration,
            new_visitors = EXCLUDED.new_visitors,
            returning_visitors = EXCLUDED.returning_visitors
        "#,
        yesterday,
    )
    .execute(&ctx.db)
    .await
    .map_err(|e| HookError::Database(e.to_string()))?;

    tracing::info!("Daily stats aggregated for {}", yesterday);
    Ok(())
}

/// Cron job: Clean up old data
pub async fn cleanup_old_data(
    ctx: CronContext,
    plugin: Arc<AnalyticsPlugin>,
) -> Result<(), HookError> {
    let config = plugin.config().await;
    let cutoff = chrono::Utc::now() - chrono::Duration::days(config.data_retention_days as i64);

    tracing::info!("Cleaning up analytics data older than {}", cutoff);

    let deleted_pageviews = sqlx::query!(
        "DELETE FROM analytics_pageviews WHERE created_at < $1",
        cutoff,
    )
    .execute(&ctx.db)
    .await
    .map_err(|e| HookError::Database(e.to_string()))?
    .rows_affected();

    let deleted_sessions = sqlx::query!(
        "DELETE FROM analytics_sessions WHERE started_at < $1",
        cutoff,
    )
    .execute(&ctx.db)
    .await
    .map_err(|e| HookError::Database(e.to_string()))?
    .rows_affected();

    let deleted_events = sqlx::query!(
        "DELETE FROM analytics_events WHERE created_at < $1",
        cutoff,
    )
    .execute(&ctx.db)
    .await
    .map_err(|e| HookError::Database(e.to_string()))?
    .rows_affected();

    tracing::info!(
        "Cleanup complete: {} pageviews, {} sessions, {} events deleted",
        deleted_pageviews,
        deleted_sessions,
        deleted_events
    );

    Ok(())
}
