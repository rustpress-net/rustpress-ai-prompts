//! RustPress Analytics Plugin
//!
//! A comprehensive analytics solution featuring:
//! - Real-time visitor tracking
//! - Page view and event analytics
//! - Geographic and device reports
//! - Session management
//! - Privacy-compliant data handling
//! - Export capabilities

pub mod api;
pub mod hooks;
pub mod models;
pub mod services;

use async_trait::async_trait;
use rustpress_plugins::prelude::*;
use services::{AnalyticsService, ReportService, TrackingService};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================
// Plugin Configuration
// ============================================

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AnalyticsConfig {
    pub tracking_enabled: bool,
    pub track_admins: bool,
    pub anonymize_ip: bool,
    pub data_retention_days: i32,
    pub excluded_ips: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub track_outbound_links: bool,
    pub track_downloads: bool,
    pub download_extensions: Vec<String>,
    pub realtime_enabled: bool,
    pub dashboard_refresh_rate: u32,
    pub default_date_range: String,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            track_admins: false,
            anonymize_ip: true,
            data_retention_days: 365,
            excluded_ips: vec![],
            excluded_paths: vec!["/admin".into(), "/api".into()],
            track_outbound_links: true,
            track_downloads: true,
            download_extensions: vec!["pdf", "zip", "doc", "docx", "xls", "xlsx"]
                .into_iter()
                .map(String::from)
                .collect(),
            realtime_enabled: true,
            dashboard_refresh_rate: 30,
            default_date_range: "30d".into(),
        }
    }
}

// ============================================
// Main Plugin Struct
// ============================================

pub struct AnalyticsPlugin {
    info: PluginInfo,
    state: RwLock<PluginState>,
    config: RwLock<AnalyticsConfig>,
    tracking_service: RwLock<Option<Arc<TrackingService>>>,
    analytics_service: RwLock<Option<Arc<AnalyticsService>>>,
    report_service: RwLock<Option<Arc<ReportService>>>,
}

impl AnalyticsPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "rustpress-analytics".into(),
                name: "RustPress Analytics".into(),
                version: "2.0.0".into(),
            },
            state: RwLock::new(PluginState::Inactive),
            config: RwLock::new(AnalyticsConfig::default()),
            tracking_service: RwLock::new(None),
            analytics_service: RwLock::new(None),
            report_service: RwLock::new(None),
        }
    }

    pub async fn config(&self) -> AnalyticsConfig {
        self.config.read().await.clone()
    }

    pub async fn tracking(&self) -> Option<Arc<TrackingService>> {
        self.tracking_service.read().await.clone()
    }

    pub async fn analytics(&self) -> Option<Arc<AnalyticsService>> {
        self.analytics_service.read().await.clone()
    }

    pub async fn reports(&self) -> Option<Arc<ReportService>> {
        self.report_service.read().await.clone()
    }

    async fn load_config(&self, settings: &SettingsManager) -> Result<AnalyticsConfig, HookError> {
        let mut config = AnalyticsConfig::default();

        if let Some(v) = settings.get("rustpress-analytics", "tracking_enabled").await? {
            config.tracking_enabled = v;
        }
        if let Some(v) = settings.get("rustpress-analytics", "track_admins").await? {
            config.track_admins = v;
        }
        if let Some(v) = settings.get("rustpress-analytics", "anonymize_ip").await? {
            config.anonymize_ip = v;
        }
        if let Some(v) = settings.get::<i32>("rustpress-analytics", "data_retention_days").await? {
            config.data_retention_days = v;
        }
        if let Some(v) = settings.get::<String>("rustpress-analytics", "excluded_ips").await? {
            config.excluded_ips = v.lines().map(String::from).collect();
        }
        if let Some(v) = settings.get::<String>("rustpress-analytics", "excluded_paths").await? {
            config.excluded_paths = v.lines().map(String::from).collect();
        }

        Ok(config)
    }
}

impl Default for AnalyticsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Lifecycle Implementation
// ============================================

#[async_trait]
impl LifecycleHook for AnalyticsPlugin {
    async fn on_activate(&self, ctx: &ActivationContext) -> Result<(), HookError> {
        tracing::info!("Activating RustPress Analytics plugin");

        // Run migrations
        ctx.run_migrations().await
            .map_err(|e| HookError::Migration(e.to_string()))?;

        // Load configuration
        let config = self.load_config(&ctx.settings).await?;
        *self.config.write().await = config.clone();

        // Initialize services
        let tracking = Arc::new(TrackingService::new(ctx.db.clone(), config.clone()));
        let analytics = Arc::new(AnalyticsService::new(ctx.db.clone(), ctx.redis.clone()));
        let reports = Arc::new(ReportService::new(ctx.db.clone()));

        *self.tracking_service.write().await = Some(tracking);
        *self.analytics_service.write().await = Some(analytics);
        *self.report_service.write().await = Some(reports);

        // Register routes
        ctx.register_routes(api::create_routes(self)).await?;

        *self.state.write().await = PluginState::Active;
        tracing::info!("RustPress Analytics activated successfully");
        Ok(())
    }

    async fn on_deactivate(&self, ctx: &DeactivationContext) -> Result<(), HookError> {
        tracing::info!("Deactivating RustPress Analytics");

        // Clear services
        *self.tracking_service.write().await = None;
        *self.analytics_service.write().await = None;
        *self.report_service.write().await = None;

        // Unregister routes
        ctx.unregister_routes().await?;

        *self.state.write().await = PluginState::Inactive;
        Ok(())
    }

    async fn on_upgrade(&self, ctx: &UpgradeContext) -> Result<(), HookError> {
        tracing::info!(
            "Upgrading Analytics from {} to {}",
            ctx.from_version,
            ctx.to_version
        );

        // Version-specific migrations
        if ctx.from_version < "2.0.0" && ctx.to_version >= "2.0.0" {
            // Migrate session data structure
            tracing::info!("Migrating to session-based tracking");
        }

        ctx.run_migrations().await?;
        Ok(())
    }

    async fn on_uninstall(&self, ctx: &UninstallContext) -> Result<(), HookError> {
        tracing::info!("Uninstalling RustPress Analytics");

        // Remove all tables
        sqlx::query("DROP TABLE IF EXISTS analytics_events CASCADE")
            .execute(&ctx.db)
            .await
            .map_err(|e| HookError::Database(e.to_string()))?;

        sqlx::query("DROP TABLE IF EXISTS analytics_pageviews CASCADE")
            .execute(&ctx.db)
            .await
            .map_err(|e| HookError::Database(e.to_string()))?;

        sqlx::query("DROP TABLE IF EXISTS analytics_sessions CASCADE")
            .execute(&ctx.db)
            .await
            .map_err(|e| HookError::Database(e.to_string()))?;

        sqlx::query("DROP TABLE IF EXISTS analytics_daily_stats CASCADE")
            .execute(&ctx.db)
            .await
            .map_err(|e| HookError::Database(e.to_string()))?;

        // Remove settings
        ctx.settings.remove_all("rustpress-analytics").await?;

        tracing::info!("RustPress Analytics uninstalled");
        Ok(())
    }
}

// ============================================
// Plugin Entry Point
// ============================================

rustpress_plugin!(AnalyticsPlugin);
