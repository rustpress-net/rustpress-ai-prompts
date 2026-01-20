# RustPress Function & Hooks Development - AI Prompt Reference

You are building functions and hooks for RustPress, a Rust-based CMS. Follow this specification exactly to create compatible hooks, actions, filters, and utility functions.

---

## HOOK SYSTEM OVERVIEW

RustPress uses a hook-based architecture for extensibility. There are three main types:

1. **Lifecycle Hooks** - Plugin/theme activation, deactivation, upgrade, uninstall
2. **Action Hooks** - Events that trigger callbacks (no return value)
3. **Filter Hooks** - Modify data through a pipeline (return modified value)

---

## HOOK REGISTRY (Core Component)

### HookRegistry Structure
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct HookRegistry {
    actions: RwLock<HashMap<String, Vec<ActionHandler>>>,
    filters: RwLock<HashMap<String, Vec<FilterHandler>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
            filters: RwLock::new(HashMap::new()),
        }
    }
}
```

### Integration with AppState
```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub config: Arc<AppConfig>,
    pub plugins: Arc<PluginRegistry>,
    pub themes: Arc<ThemeManager>,
    pub hooks: Arc<HookRegistry>,    // Hook registry
}
```

---

## HOOK PRIORITIES

Priorities determine execution order. Higher values run first.

| Constant | Value | Use Case |
|----------|-------|----------|
| `HIGHEST` | 100 | Security checks, validation |
| `HIGH` | 50 | Early processing |
| `NORMAL` | 0 | Default priority |
| `LOW` | -50 | Late processing |
| `LOWEST` | -100 | Cleanup, final modifications |

```rust
pub mod priority {
    pub const HIGHEST: i32 = 100;
    pub const HIGH: i32 = 50;
    pub const NORMAL: i32 = 0;
    pub const LOW: i32 = -50;
    pub const LOWEST: i32 = -100;
}
```

---

## LIFECYCLE HOOKS

### Configuration in plugin.toml
```toml
[hooks]
activate = "activate_plugin"
deactivate = "deactivate_plugin"
uninstall = "uninstall_plugin"
```

### Lifecycle Hook Trait
```rust
use async_trait::async_trait;

#[async_trait]
pub trait LifecycleHook: Send + Sync {
    /// Called when plugin is activated
    async fn on_activate(&self, ctx: &ActivationContext) -> Result<(), HookError>;

    /// Called when plugin is deactivated
    async fn on_deactivate(&self, ctx: &DeactivationContext) -> Result<(), HookError>;

    /// Called when plugin version changes
    async fn on_upgrade(&self, ctx: &UpgradeContext) -> Result<(), HookError>;

    /// Called when plugin is completely removed
    async fn on_uninstall(&self, ctx: &UninstallContext) -> Result<(), HookError>;
}
```

### Context Objects
```rust
/// Activation context
pub struct ActivationContext {
    pub db: PgPool,
    pub settings: SettingsManager,
    pub previous_version: Option<String>,
}

impl ActivationContext {
    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<(), MigrationError> {
        // Executes migrations from migrations/ directory
    }

    /// Register plugin routes
    pub async fn register_routes(&self, routes: Router) -> Result<(), RouteError> {
        // Adds routes to the main router
    }
}

/// Deactivation context
pub struct DeactivationContext {
    pub db: PgPool,
    pub settings: SettingsManager,
}

impl DeactivationContext {
    /// Unregister plugin routes
    pub async fn unregister_routes(&self) -> Result<(), RouteError> {
        // Removes plugin routes from router
    }
}

/// Upgrade context
pub struct UpgradeContext {
    pub db: PgPool,
    pub settings: SettingsManager,
    pub from_version: String,
    pub to_version: String,
}

/// Uninstall context
pub struct UninstallContext {
    pub db: PgPool,
    pub settings: SettingsManager,
    pub fs: FileSystem,
}

impl UninstallContext {
    /// Remove all plugin settings
    pub async fn remove_all_settings(&self, namespace: &str) -> Result<(), SettingsError> {
        self.settings.remove_all(namespace).await
    }
}
```

### Complete Lifecycle Implementation
```rust
pub struct MyPlugin {
    info: PluginInfo,
    state: RwLock<PluginState>,
    config: RwLock<Option<MyConfig>>,
}

#[async_trait]
impl LifecycleHook for MyPlugin {
    async fn on_activate(&self, ctx: &ActivationContext) -> Result<(), HookError> {
        // 1. Run database migrations
        ctx.run_migrations().await
            .map_err(|e| HookError::Migration(e.to_string()))?;

        // 2. Load and validate configuration
        let config = MyConfig::load(&ctx.settings).await
            .map_err(|e| HookError::Config(e.to_string()))?;

        // 3. Validate required settings
        if config.api_key.is_empty() {
            return Err(HookError::Config("API key is required".into()));
        }

        // 4. Initialize external connections
        self.init_external_service(&config).await?;

        // 5. Store config
        *self.config.write().await = Some(config);

        // 6. Register routes
        ctx.register_routes(self.create_routes()).await?;

        // 7. Update state
        *self.state.write().await = PluginState::Active;

        tracing::info!("Plugin {} activated successfully", self.info.id);
        Ok(())
    }

    async fn on_deactivate(&self, ctx: &DeactivationContext) -> Result<(), HookError> {
        // 1. Close external connections
        self.close_external_service().await?;

        // 2. Clear caches
        self.clear_caches().await?;

        // 3. Unregister routes
        ctx.unregister_routes().await?;

        // 4. Update state
        *self.state.write().await = PluginState::Inactive;

        tracing::info!("Plugin {} deactivated", self.info.id);
        Ok(())
    }

    async fn on_upgrade(&self, ctx: &UpgradeContext) -> Result<(), HookError> {
        let from = &ctx.from_version;
        let to = &ctx.to_version;

        tracing::info!("Upgrading plugin from {} to {}", from, to);

        // Version-specific migrations
        if from < "1.1.0" && to >= "1.1.0" {
            self.migrate_to_1_1_0(ctx).await?;
        }

        if from < "2.0.0" && to >= "2.0.0" {
            self.migrate_to_2_0_0(ctx).await?;
        }

        // Always run general migrations
        ctx.run_migrations().await?;

        Ok(())
    }

    async fn on_uninstall(&self, ctx: &UninstallContext) -> Result<(), HookError> {
        tracing::info!("Uninstalling plugin {}", self.info.id);

        // 1. Remove database tables
        sqlx::query("DROP TABLE IF EXISTS my_plugin_data CASCADE")
            .execute(&ctx.db)
            .await
            .map_err(|e| HookError::Database(e.to_string()))?;

        // 2. Remove all settings
        ctx.remove_all_settings("my-plugin").await?;

        // 3. Remove uploaded files
        ctx.fs.remove_dir_all("uploads/my-plugin").await
            .map_err(|e| HookError::FileSystem(e.to_string()))?;

        // 4. Clear all caches
        self.clear_all_caches().await?;

        tracing::info!("Plugin {} uninstalled successfully", self.info.id);
        Ok(())
    }
}
```

### Plugin States
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Discovered,    // Found but not loaded
    Inactive,      // Loaded but not active
    Activating,    // In activation process
    Active,        // Fully operational
    Deactivating,  // In deactivation process
    Uninstalling,  // Being removed
    Error(String), // Failed state
}
```

### State Transitions
```
Discovered → Inactive → Activating → Active
                ↑                       ↓
                └── Deactivating ←──────┘
                         ↓
                    Uninstalling
```

---

## ACTION HOOKS

Actions are events that trigger callbacks. They don't return values.

### Configuration in plugin.toml
```toml
[[hooks.actions]]
hook = "post_publish"
callback = "on_post_published"
priority = 10

[[hooks.actions]]
hook = "user_login"
callback = "on_user_login"
priority = 0

[[hooks.actions]]
hook = "comment_create"
callback = "on_comment_created"
priority = -10
```

### Action Handler Signature
```rust
pub type ActionHandler = Arc<dyn Fn(&ActionContext, &dyn Any) -> BoxFuture<'static, Result<(), HookError>> + Send + Sync>;
```

### ActionContext
```rust
pub struct ActionContext {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub user: Option<User>,
    pub request_id: String,
    pub plugin_id: String,
}
```

### Registering Action Hooks
```rust
impl HookRegistry {
    /// Register an action handler
    pub async fn add_action<F, Fut>(
        &self,
        hook_name: &str,
        callback: F,
        priority: i32,
    ) where
        F: Fn(ActionContext, Box<dyn Any + Send>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), HookError>> + Send + 'static,
    {
        let mut actions = self.actions.write().await;
        let handlers = actions.entry(hook_name.to_string()).or_insert_with(Vec::new);

        handlers.push(ActionHandler {
            callback: Arc::new(move |ctx, data| Box::pin(callback(ctx.clone(), data))),
            priority,
        });

        // Sort by priority (highest first)
        handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Execute all handlers for an action
    pub async fn do_action<T: Any + Send + Clone>(
        &self,
        hook_name: &str,
        ctx: &ActionContext,
        data: T,
    ) -> Result<(), HookError> {
        let actions = self.actions.read().await;

        if let Some(handlers) = actions.get(hook_name) {
            for handler in handlers {
                (handler.callback)(ctx, Box::new(data.clone())).await?;
            }
        }

        Ok(())
    }
}
```

### Example Action Handlers
```rust
/// Called when a post is published
pub async fn on_post_published(
    ctx: ActionContext,
    data: Box<dyn Any + Send>,
) -> Result<(), HookError> {
    let post = data.downcast_ref::<Post>()
        .ok_or(HookError::InvalidData("Expected Post"))?;

    // Send notification
    send_publish_notification(&ctx, post).await?;

    // Update search index
    update_search_index(&ctx, post).await?;

    // Invalidate caches
    invalidate_post_caches(&ctx, post.id).await?;

    tracing::info!("Post {} published, notifications sent", post.id);
    Ok(())
}

/// Called when a user logs in
pub async fn on_user_login(
    ctx: ActionContext,
    data: Box<dyn Any + Send>,
) -> Result<(), HookError> {
    let user = data.downcast_ref::<User>()
        .ok_or(HookError::InvalidData("Expected User"))?;

    // Log login event
    sqlx::query!(
        "INSERT INTO login_log (user_id, ip_address, user_agent, created_at) VALUES ($1, $2, $3, NOW())",
        user.id,
        ctx.ip_address,
        ctx.user_agent
    )
    .execute(&ctx.db)
    .await?;

    // Update last login
    sqlx::query!(
        "UPDATE users SET last_login = NOW() WHERE id = $1",
        user.id
    )
    .execute(&ctx.db)
    .await?;

    Ok(())
}

/// Called when a comment is created
pub async fn on_comment_created(
    ctx: ActionContext,
    data: Box<dyn Any + Send>,
) -> Result<(), HookError> {
    let comment = data.downcast_ref::<Comment>()
        .ok_or(HookError::InvalidData("Expected Comment"))?;

    // Notify post author
    notify_post_author(&ctx, comment).await?;

    // Update comment count
    sqlx::query!(
        "UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1",
        comment.post_id
    )
    .execute(&ctx.db)
    .await?;

    Ok(())
}
```

### Common Action Hooks
| Hook Name | Data Type | Description |
|-----------|-----------|-------------|
| `init` | `()` | Application initialized |
| `post_create` | `Post` | Post created |
| `post_update` | `Post` | Post updated |
| `post_delete` | `i64` | Post deleted (ID) |
| `post_publish` | `Post` | Post published |
| `post_unpublish` | `Post` | Post unpublished |
| `user_register` | `User` | User registered |
| `user_login` | `User` | User logged in |
| `user_logout` | `User` | User logged out |
| `user_update` | `User` | User profile updated |
| `comment_create` | `Comment` | Comment created |
| `comment_approve` | `Comment` | Comment approved |
| `comment_delete` | `i64` | Comment deleted (ID) |
| `media_upload` | `Media` | Media file uploaded |
| `media_delete` | `i64` | Media deleted (ID) |
| `settings_update` | `SettingsChange` | Settings changed |
| `plugin_activate` | `String` | Plugin activated (ID) |
| `plugin_deactivate` | `String` | Plugin deactivated (ID) |
| `theme_switch` | `ThemeChange` | Theme switched |
| `cron_run` | `String` | Cron job executed |

---

## FILTER HOOKS

Filters modify data through a pipeline. Each handler receives data and returns modified data.

### Configuration in plugin.toml
```toml
[[hooks.filters]]
hook = "content_filter"
callback = "filter_content"
priority = 20

[[hooks.filters]]
hook = "post_title"
callback = "filter_post_title"
priority = 0

[[hooks.filters]]
hook = "excerpt_length"
callback = "filter_excerpt_length"
priority = 10
```

### Filter Handler Signature
```rust
pub type FilterHandler<T> = Arc<dyn Fn(&FilterContext, T) -> BoxFuture<'static, Result<T, HookError>> + Send + Sync>;
```

### FilterContext
```rust
pub struct FilterContext {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub user: Option<User>,
    pub settings: SettingsManager,
    pub plugin_id: String,
}
```

### Registering Filter Hooks
```rust
impl HookRegistry {
    /// Register a filter handler
    pub async fn add_filter<T, F, Fut>(
        &self,
        hook_name: &str,
        callback: F,
        priority: i32,
    ) where
        T: Clone + Send + 'static,
        F: Fn(FilterContext, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, HookError>> + Send + 'static,
    {
        let mut filters = self.filters.write().await;
        let handlers = filters.entry(hook_name.to_string()).or_insert_with(Vec::new);

        handlers.push(FilterHandler {
            callback: Arc::new(move |ctx, value| Box::pin(callback(ctx.clone(), value))),
            priority,
        });

        // Sort by priority (highest first)
        handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Apply all filters to a value
    pub async fn apply_filters<T: Clone + Send + 'static>(
        &self,
        hook_name: &str,
        ctx: &FilterContext,
        value: T,
    ) -> Result<T, HookError> {
        let filters = self.filters.read().await;
        let mut result = value;

        if let Some(handlers) = filters.get(hook_name) {
            for handler in handlers {
                result = (handler.callback)(ctx, result).await?;
            }
        }

        Ok(result)
    }
}
```

### Example Filter Handlers
```rust
/// Filter post content before display
pub async fn filter_content(
    ctx: FilterContext,
    content: String,
) -> Result<String, HookError> {
    let mut result = content;

    // Convert shortcodes to HTML
    result = process_shortcodes(&ctx, &result).await?;

    // Add rel="nofollow" to external links
    result = add_nofollow_to_external_links(&result);

    // Lazy load images
    result = add_lazy_loading(&result);

    Ok(result)
}

/// Filter post title
pub async fn filter_post_title(
    _ctx: FilterContext,
    title: String,
) -> Result<String, HookError> {
    // Remove HTML tags
    let clean = strip_tags(&title);

    // Decode HTML entities
    let decoded = html_escape::decode_html_entities(&clean);

    // Trim whitespace
    Ok(decoded.trim().to_string())
}

/// Filter excerpt length
pub async fn filter_excerpt_length(
    ctx: FilterContext,
    length: usize,
) -> Result<usize, HookError> {
    // Check if custom length is set
    if let Some(custom) = ctx.settings.get::<usize>("excerpt_length").await? {
        return Ok(custom);
    }

    // Default to provided length
    Ok(length)
}

/// Filter post query arguments
pub async fn filter_post_query(
    ctx: FilterContext,
    mut query: PostQuery,
) -> Result<PostQuery, HookError> {
    // Exclude certain categories for non-admins
    if !ctx.user.as_ref().map(|u| u.is_admin()).unwrap_or(false) {
        query.exclude_categories.push("private");
    }

    // Add featured posts to top
    if query.include_featured {
        query.order_by_featured = true;
    }

    Ok(query)
}

/// Filter upload allowed file types
pub async fn filter_allowed_mime_types(
    _ctx: FilterContext,
    mut types: Vec<String>,
) -> Result<Vec<String>, HookError> {
    // Add SVG support
    types.push("image/svg+xml".to_string());

    // Add WebP support
    types.push("image/webp".to_string());

    Ok(types)
}

/// Filter maximum upload size
pub async fn filter_max_upload_size(
    ctx: FilterContext,
    size: usize,
) -> Result<usize, HookError> {
    // Admins get larger limit
    if ctx.user.as_ref().map(|u| u.is_admin()).unwrap_or(false) {
        return Ok(50 * 1024 * 1024); // 50MB
    }

    // Regular users
    Ok(size.min(10 * 1024 * 1024)) // Max 10MB
}
```

### Common Filter Hooks
| Hook Name | Type | Description |
|-----------|------|-------------|
| `content_filter` | `String` | Post/page content |
| `excerpt_filter` | `String` | Post excerpt |
| `title_filter` | `String` | Post/page title |
| `post_query` | `PostQuery` | Query parameters |
| `post_data` | `PostData` | Post before save |
| `comment_text` | `String` | Comment content |
| `user_data` | `UserData` | User before save |
| `upload_dir` | `String` | Upload directory path |
| `allowed_mime_types` | `Vec<String>` | Allowed upload types |
| `max_upload_size` | `usize` | Max file size |
| `redirect_url` | `String` | Redirect destination |
| `login_redirect` | `String` | Post-login redirect |
| `logout_redirect` | `String` | Post-logout redirect |
| `admin_menu` | `Vec<MenuItem>` | Admin menu items |
| `nav_menu` | `Vec<MenuItem>` | Navigation menu |
| `widget_output` | `String` | Widget HTML |
| `email_subject` | `String` | Email subject |
| `email_body` | `String` | Email content |
| `seo_title` | `String` | SEO page title |
| `seo_description` | `String` | SEO description |

---

## ERROR HANDLING

### HookError Enum
```rust
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Invalid data: {0}")]
    InvalidData(&'static str),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Hook aborted: {0}")]
    Aborted(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for HookError {
    fn from(e: sqlx::Error) -> Self {
        HookError::Database(e.to_string())
    }
}

impl From<std::io::Error> for HookError {
    fn from(e: std::io::Error) -> Self {
        HookError::FileSystem(e.to_string())
    }
}
```

---

## UTILITY FUNCTIONS

### Settings Access
```rust
pub struct SettingsManager {
    db: PgPool,
    cache: deadpool_redis::Pool,
}

impl SettingsManager {
    /// Get a setting value
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SettingsError> {
        // Try cache first
        if let Some(value) = self.get_cached::<T>(key).await? {
            return Ok(Some(value));
        }

        // Query database
        let row = sqlx::query!(
            "SELECT value FROM settings WHERE key = $1",
            key
        )
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(r) => {
                let value: T = serde_json::from_str(&r.value)?;
                self.set_cache(key, &value).await?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a setting value
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), SettingsError> {
        let json = serde_json::to_string(value)?;

        sqlx::query!(
            r#"
            INSERT INTO settings (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
            "#,
            key,
            json
        )
        .execute(&self.db)
        .await?;

        // Update cache
        self.set_cache(key, value).await?;

        Ok(())
    }

    /// Remove a setting
    pub async fn remove(&self, key: &str) -> Result<(), SettingsError> {
        sqlx::query!("DELETE FROM settings WHERE key = $1", key)
            .execute(&self.db)
            .await?;

        self.invalidate_cache(key).await?;
        Ok(())
    }

    /// Remove all settings for a namespace
    pub async fn remove_all(&self, namespace: &str) -> Result<(), SettingsError> {
        sqlx::query!(
            "DELETE FROM settings WHERE key LIKE $1",
            format!("{}:%", namespace)
        )
        .execute(&self.db)
        .await?;

        self.invalidate_cache_pattern(&format!("settings:{}:*", namespace)).await?;
        Ok(())
    }
}
```

### Cache Utilities
```rust
/// Get cached value with automatic JSON deserialization
pub async fn cache_get<T: DeserializeOwned>(
    redis: &deadpool_redis::Pool,
    key: &str,
) -> Result<Option<T>, CacheError> {
    let mut conn = redis.get().await?;

    let value: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(&mut conn)
        .await?;

    match value {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Set cached value with TTL
pub async fn cache_set<T: Serialize>(
    redis: &deadpool_redis::Pool,
    key: &str,
    value: &T,
    ttl_secs: u64,
) -> Result<(), CacheError> {
    let mut conn = redis.get().await?;
    let json = serde_json::to_string(value)?;

    redis::cmd("SETEX")
        .arg(key)
        .arg(ttl_secs)
        .arg(json)
        .query_async(&mut conn)
        .await?;

    Ok(())
}

/// Delete cached value
pub async fn cache_delete(
    redis: &deadpool_redis::Pool,
    key: &str,
) -> Result<(), CacheError> {
    let mut conn = redis.get().await?;
    redis::cmd("DEL").arg(key).query_async(&mut conn).await?;
    Ok(())
}

/// Delete cached values by pattern
pub async fn cache_delete_pattern(
    redis: &deadpool_redis::Pool,
    pattern: &str,
) -> Result<(), CacheError> {
    let mut conn = redis.get().await?;

    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(pattern)
        .query_async(&mut conn)
        .await?;

    if !keys.is_empty() {
        redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut conn)
            .await?;
    }

    Ok(())
}

/// Cache-aside pattern helper
pub async fn cache_remember<T, F, Fut>(
    redis: &deadpool_redis::Pool,
    key: &str,
    ttl_secs: u64,
    fetch: F,
) -> Result<T, CacheError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, CacheError>>,
{
    // Try cache first
    if let Some(value) = cache_get(redis, key).await? {
        return Ok(value);
    }

    // Fetch from source
    let value = fetch().await?;

    // Store in cache
    cache_set(redis, key, &value, ttl_secs).await?;

    Ok(value)
}
```

### Slug Generation
```rust
use slug::slugify;

pub fn generate_slug(text: &str) -> String {
    slugify(text)
}

pub fn generate_unique_slug(text: &str, existing: &[String]) -> String {
    let base = generate_slug(text);

    if !existing.contains(&base) {
        return base;
    }

    let mut counter = 2;
    loop {
        let candidate = format!("{}-{}", base, counter);
        if !existing.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}
```

### HTML Processing
```rust
use ammonia::Builder;

/// Sanitize HTML content
pub fn sanitize_html(html: &str) -> String {
    Builder::default()
        .tags(hashset!["p", "br", "a", "strong", "em", "ul", "ol", "li", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "code", "pre", "img"])
        .link_rel(Some("nofollow noopener"))
        .url_relative(ammonia::UrlRelative::PassThrough)
        .clean(html)
        .to_string()
}

/// Strip all HTML tags
pub fn strip_tags(html: &str) -> String {
    Builder::default()
        .tags(hashset![])
        .clean(html)
        .to_string()
}

/// Generate excerpt from content
pub fn generate_excerpt(content: &str, length: usize) -> String {
    let text = strip_tags(content);
    let trimmed = text.trim();

    if trimmed.len() <= length {
        return trimmed.to_string();
    }

    // Find word boundary
    let truncated = &trimmed[..length];
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &truncated[..last_space])
    } else {
        format!("{}...", truncated)
    }
}
```

### Email Utilities
```rust
use lettre::{Message, SmtpTransport, Transport};

pub struct EmailService {
    transport: SmtpTransport,
    from_address: String,
    from_name: String,
}

impl EmailService {
    /// Send an email
    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EmailError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_address).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_string())?;

        self.transport.send(&email)?;
        Ok(())
    }

    /// Send templated email
    pub async fn send_template(
        &self,
        to: &str,
        template: &str,
        context: &tera::Context,
    ) -> Result<(), EmailError> {
        let tera = tera::Tera::new("templates/emails/*")?;
        let body = tera.render(template, context)?;
        let subject = tera.render(&format!("{}_subject", template), context)?;

        self.send(to, &subject, &body).await
    }
}
```

---

## BEST PRACTICES

1. **Always implement all lifecycle hooks** - Even empty ones for future extensibility
2. **Use appropriate priorities** - Security checks highest, cleanup lowest
3. **Handle errors gracefully** - Return `Result`, never panic
4. **Clean up on deactivate/uninstall** - Close connections, clear caches
5. **Use typed data in hooks** - Avoid `Box<dyn Any>` when possible
6. **Document custom hooks** - Specify data types and expected behavior
7. **Test hook interactions** - Multiple plugins may use same hooks
8. **Log important events** - Use tracing for debugging
9. **Cache expensive operations** - Filters run frequently
10. **Keep handlers fast** - Async for I/O, avoid blocking

---

## HOOK EXECUTION ORDER

```
Request Flow:
1. init (action)
2. route_matched (action)
3. auth_check (filter) - modify auth result
4. permission_check (filter) - modify permissions
5. pre_query (filter) - modify query params
6. query_execute (action)
7. post_query (filter) - modify results
8. content_filter (filter) - modify content
9. response_headers (filter) - modify headers
10. response_send (action)

Post Save Flow:
1. pre_save (filter) - validate/modify data
2. save (action) - after database write
3. post_save (action) - cleanup, notifications
4. cache_invalidate (action) - clear caches
```

---

## CREATING CUSTOM HOOKS

### Define Hook in Plugin
```rust
impl MyPlugin {
    /// Execute custom action hook
    pub async fn do_my_action(&self, ctx: &ActionContext, data: MyData) -> Result<(), HookError> {
        ctx.hooks.do_action("my_plugin/custom_action", ctx, data).await
    }

    /// Apply custom filter hook
    pub async fn apply_my_filter(&self, ctx: &FilterContext, value: String) -> Result<String, HookError> {
        ctx.hooks.apply_filters("my_plugin/custom_filter", ctx, value).await
    }
}
```

### Document Custom Hooks
```rust
/// # Custom Hooks
///
/// ## Actions
/// - `my_plugin/custom_action` - Triggered when X happens
///   - Data: `MyData { id: i64, value: String }`
///   - Use case: React to custom events
///
/// ## Filters
/// - `my_plugin/custom_filter` - Modifies Y data
///   - Type: `String`
///   - Use case: Customize output format
```
