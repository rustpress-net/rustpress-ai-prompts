//! Sample RustPress Hooks & Functions
//!
//! A minimal example demonstrating:
//! - Hook registry setup
//! - Action hooks (events)
//! - Filter hooks (data transformation)
//! - Lifecycle hooks
//! - Utility functions

use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================
// Types
// ============================================

/// Hook execution priority
pub mod priority {
    pub const HIGHEST: i32 = 100;
    pub const HIGH: i32 = 50;
    pub const NORMAL: i32 = 0;
    pub const LOW: i32 = -50;
    pub const LOWEST: i32 = -100;
}

/// Hook errors
#[derive(Debug)]
pub enum HookError {
    Internal(String),
    InvalidData(&'static str),
}

/// Action handler type
type ActionFn = Arc<
    dyn Fn(ActionContext, Box<dyn Any + Send>) -> Pin<Box<dyn Future<Output = Result<(), HookError>> + Send>>
        + Send
        + Sync,
>;

/// Filter handler type
type FilterFn<T> = Arc<
    dyn Fn(FilterContext, T) -> Pin<Box<dyn Future<Output = Result<T, HookError>> + Send>>
        + Send
        + Sync,
>;

// ============================================
// Context Objects
// ============================================

#[derive(Clone)]
pub struct ActionContext {
    pub request_id: String,
    pub user_id: Option<i64>,
}

#[derive(Clone)]
pub struct FilterContext {
    pub request_id: String,
    pub user_id: Option<i64>,
}

// ============================================
// Hook Registry
// ============================================

struct ActionHandler {
    callback: ActionFn,
    priority: i32,
}

struct FilterHandler<T> {
    callback: FilterFn<T>,
    priority: i32,
}

/// Simple hook registry for actions and string filters
pub struct HookRegistry {
    actions: RwLock<HashMap<String, Vec<ActionHandler>>>,
    string_filters: RwLock<HashMap<String, Vec<FilterHandler<String>>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
            string_filters: RwLock::new(HashMap::new()),
        }
    }

    /// Register an action hook
    pub async fn add_action<F, Fut>(&self, hook: &str, callback: F, priority: i32)
    where
        F: Fn(ActionContext, Box<dyn Any + Send>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), HookError>> + Send + 'static,
    {
        let mut actions = self.actions.write().await;
        let handlers = actions.entry(hook.to_string()).or_default();

        handlers.push(ActionHandler {
            callback: Arc::new(move |ctx, data| Box::pin(callback(ctx, data))),
            priority,
        });

        handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Execute an action hook
    pub async fn do_action<T: Any + Send + Clone + 'static>(
        &self,
        hook: &str,
        ctx: &ActionContext,
        data: T,
    ) -> Result<(), HookError> {
        let actions = self.actions.read().await;

        if let Some(handlers) = actions.get(hook) {
            for handler in handlers {
                (handler.callback)(ctx.clone(), Box::new(data.clone())).await?;
            }
        }

        Ok(())
    }

    /// Register a string filter hook
    pub async fn add_filter<F, Fut>(&self, hook: &str, callback: F, priority: i32)
    where
        F: Fn(FilterContext, String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, HookError>> + Send + 'static,
    {
        let mut filters = self.string_filters.write().await;
        let handlers = filters.entry(hook.to_string()).or_default();

        handlers.push(FilterHandler {
            callback: Arc::new(move |ctx, value| Box::pin(callback(ctx, value))),
            priority,
        });

        handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Apply string filters
    pub async fn apply_filters(
        &self,
        hook: &str,
        ctx: &FilterContext,
        value: String,
    ) -> Result<String, HookError> {
        let filters = self.string_filters.read().await;
        let mut result = value;

        if let Some(handlers) = filters.get(hook) {
            for handler in handlers {
                result = (handler.callback)(ctx.clone(), result).await?;
            }
        }

        Ok(result)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Sample Action Handlers
// ============================================

/// Log when a post is published
pub async fn on_post_published(
    ctx: ActionContext,
    data: Box<dyn Any + Send>,
) -> Result<(), HookError> {
    if let Some(post_id) = data.downcast_ref::<i64>() {
        tracing::info!(
            request_id = %ctx.request_id,
            post_id = %post_id,
            "Post published"
        );
    }
    Ok(())
}

/// Log when a user logs in
pub async fn on_user_login(
    ctx: ActionContext,
    data: Box<dyn Any + Send>,
) -> Result<(), HookError> {
    if let Some(user_id) = data.downcast_ref::<i64>() {
        tracing::info!(
            request_id = %ctx.request_id,
            user_id = %user_id,
            "User logged in"
        );
    }
    Ok(())
}

// ============================================
// Sample Filter Handlers
// ============================================

/// Add "nofollow" to external links
pub async fn filter_add_nofollow(
    _ctx: FilterContext,
    content: String,
) -> Result<String, HookError> {
    // Simple example - in real impl use HTML parser
    Ok(content.replace(
        r#"<a href="http"#,
        r#"<a rel="nofollow" href="http"#,
    ))
}

/// Convert text to uppercase (example)
pub async fn filter_uppercase(
    _ctx: FilterContext,
    content: String,
) -> Result<String, HookError> {
    Ok(content.to_uppercase())
}

/// Add a prefix to content
pub async fn filter_add_prefix(
    _ctx: FilterContext,
    content: String,
) -> Result<String, HookError> {
    Ok(format!("[FILTERED] {}", content))
}

// ============================================
// Lifecycle Trait
// ============================================

#[async_trait]
pub trait LifecycleHook: Send + Sync {
    async fn on_activate(&self) -> Result<(), HookError>;
    async fn on_deactivate(&self) -> Result<(), HookError>;
}

/// Example component implementing lifecycle
pub struct MyComponent {
    name: String,
    active: RwLock<bool>,
}

impl MyComponent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active: RwLock::new(false),
        }
    }

    pub async fn is_active(&self) -> bool {
        *self.active.read().await
    }
}

#[async_trait]
impl LifecycleHook for MyComponent {
    async fn on_activate(&self) -> Result<(), HookError> {
        tracing::info!("Activating component: {}", self.name);
        *self.active.write().await = true;
        Ok(())
    }

    async fn on_deactivate(&self) -> Result<(), HookError> {
        tracing::info!("Deactivating component: {}", self.name);
        *self.active.write().await = false;
        Ok(())
    }
}

// ============================================
// Usage Example
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_action_hooks() {
        let registry = HookRegistry::new();

        // Register action
        registry
            .add_action("post_publish", on_post_published, priority::NORMAL)
            .await;

        // Execute action
        let ctx = ActionContext {
            request_id: "test-123".into(),
            user_id: Some(1),
        };

        let result = registry.do_action("post_publish", &ctx, 42i64).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filter_hooks() {
        let registry = HookRegistry::new();

        // Register filters (executed in priority order)
        registry
            .add_filter("content", filter_add_prefix, priority::HIGH)
            .await;
        registry
            .add_filter("content", filter_uppercase, priority::NORMAL)
            .await;

        let ctx = FilterContext {
            request_id: "test-456".into(),
            user_id: None,
        };

        let result = registry
            .apply_filters("content", &ctx, "hello world".into())
            .await
            .unwrap();

        // High priority runs first, then normal
        assert_eq!(result, "[FILTERED] HELLO WORLD");
    }

    #[tokio::test]
    async fn test_lifecycle() {
        let component = MyComponent::new("test");

        assert!(!component.is_active().await);

        component.on_activate().await.unwrap();
        assert!(component.is_active().await);

        component.on_deactivate().await.unwrap();
        assert!(!component.is_active().await);
    }
}
