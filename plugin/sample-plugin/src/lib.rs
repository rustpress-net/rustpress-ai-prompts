//! Hello World Plugin for RustPress
//!
//! A minimal example plugin demonstrating:
//! - Plugin structure and manifest
//! - Lifecycle hooks (activate/deactivate)
//! - REST API endpoints
//! - Settings management
//! - Shortcode rendering

use async_trait::async_trait;
use axum::{extract::State, Json};
use chrono::Utc;
use rustpress_plugins::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================
// Plugin Struct
// ============================================

pub struct HelloWorldPlugin {
    info: PluginInfo,
    state: RwLock<PluginState>,
    greeting: RwLock<String>,
}

impl HelloWorldPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "hello-world".into(),
                name: "Hello World".into(),
                version: "1.0.0".into(),
            },
            state: RwLock::new(PluginState::Inactive),
            greeting: RwLock::new("Hello, World!".into()),
        }
    }
}

impl Default for HelloWorldPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Lifecycle Hooks
// ============================================

#[async_trait]
impl LifecycleHook for HelloWorldPlugin {
    async fn on_activate(&self, ctx: &ActivationContext) -> Result<(), HookError> {
        tracing::info!("Activating Hello World plugin");

        // Load greeting from settings
        if let Some(greeting) = ctx.settings.get::<String>("hello-world", "greeting").await? {
            *self.greeting.write().await = greeting;
        }

        *self.state.write().await = PluginState::Active;
        tracing::info!("Hello World plugin activated!");
        Ok(())
    }

    async fn on_deactivate(&self, _ctx: &DeactivationContext) -> Result<(), HookError> {
        tracing::info!("Deactivating Hello World plugin");
        *self.state.write().await = PluginState::Inactive;
        Ok(())
    }

    async fn on_upgrade(&self, ctx: &UpgradeContext) -> Result<(), HookError> {
        tracing::info!(
            "Upgrading Hello World from {} to {}",
            ctx.from_version,
            ctx.to_version
        );
        Ok(())
    }

    async fn on_uninstall(&self, ctx: &UninstallContext) -> Result<(), HookError> {
        tracing::info!("Uninstalling Hello World plugin");
        ctx.settings.remove_all("hello-world").await?;
        Ok(())
    }
}

// ============================================
// API Types
// ============================================

#[derive(Serialize)]
pub struct GreetingResponse {
    message: String,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
pub struct SetGreetingRequest {
    message: String,
}

// ============================================
// API Handlers
// ============================================

/// GET /api/v1/hello-world/greet
pub async fn get_greeting(
    State(plugin): State<Arc<HelloWorldPlugin>>,
) -> Json<GreetingResponse> {
    let greeting = plugin.greeting.read().await.clone();
    let show_date = true; // Would come from settings

    Json(GreetingResponse {
        message: greeting,
        timestamp: if show_date {
            Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string())
        } else {
            None
        },
    })
}

/// POST /api/v1/hello-world/greet
pub async fn set_greeting(
    State(plugin): State<Arc<HelloWorldPlugin>>,
    Json(input): Json<SetGreetingRequest>,
) -> Json<GreetingResponse> {
    *plugin.greeting.write().await = input.message.clone();

    Json(GreetingResponse {
        message: input.message,
        timestamp: Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()),
    })
}

// ============================================
// Shortcode Handler
// ============================================

/// Renders [hello] or [hello name="User"]
pub fn render_hello(attrs: &ShortcodeAttributes, _content: Option<&str>) -> String {
    let name = attrs
        .get("name")
        .map(|s| s.as_str())
        .unwrap_or("World");

    format!(r#"<div class="hello-greeting">Hello, {}!</div>"#, name)
}

// ============================================
// Plugin Entry Point
// ============================================

rustpress_plugin!(HelloWorldPlugin);
