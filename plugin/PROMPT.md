# RustPress Plugin Development - AI Prompt Reference

You are building a plugin for RustPress, a Rust-based CMS. Follow this specification exactly to create compatible plugins.

---

## PLUGIN MANIFEST (`plugin.toml`)

Every plugin MUST have a `plugin.toml` manifest file at the root.

### Minimal Required Configuration
```toml
[plugin]
id = "my-plugin"           # Unique identifier (kebab-case)
name = "My Plugin"         # Display name
version = "1.0.0"          # Semantic version
description = "Description"
author = "Author Name"
```

### Complete Configuration Schema
```toml
[plugin]
id = "my-plugin"
name = "My Plugin"
version = "1.0.0"
description = "Plugin description"
author = "Author Name"
author_url = "https://example.com"
license = "MIT"
min_rustpress_version = "1.0.0"
tags = ["utility", "tools"]
category = "utility"       # utility|social|marketing|business|media|security|performance
icon = "icon.png"

# ============================================
# DEPENDENCIES
# ============================================
[dependencies.plugins]
"required-plugin" = "^1.0"     # Requires another plugin

[dependencies.conflicts]
"incompatible-plugin" = "*"    # Cannot coexist with this plugin

# ============================================
# SETTINGS SCHEMA
# ============================================
# Each setting follows this pattern:
# [settings.schema.{setting_key}]
# setting_type = "type"
# label = "Display Label"
# default = default_value
# required = true|false
# options = ["opt1", "opt2"]  # For select/multiselect types

[settings.schema.enable_feature]
setting_type = "boolean"
label = "Enable Feature"
default = true
required = false

[settings.schema.api_key]
setting_type = "password"
label = "API Key"
required = true

[settings.schema.mode]
setting_type = "select"
label = "Mode"
options = ["basic", "advanced", "expert"]
default = "basic"

[settings.schema.max_items]
setting_type = "integer"
label = "Maximum Items"
default = 100

[settings.schema.webhook_url]
setting_type = "url"
label = "Webhook URL"
required = false

[settings.schema.primary_color]
setting_type = "color"
label = "Primary Color"
default = "#007bff"

[settings.schema.logo_image]
setting_type = "image"
label = "Logo"

[settings.schema.custom_code]
setting_type = "code"
label = "Custom CSS"
default = ""

# ============================================
# LIFECYCLE HOOKS
# ============================================
[hooks]
activate = "activate_plugin"       # Called on plugin activation
deactivate = "deactivate_plugin"   # Called on plugin deactivation
uninstall = "uninstall_plugin"     # Called on plugin removal

# Action hooks - respond to events
[[hooks.actions]]
hook = "post_publish"              # Event name to listen for
callback = "on_post_published"     # Handler function name
priority = 10                      # -100 to 100 (higher runs first)

[[hooks.actions]]
hook = "user_login"
callback = "on_user_login"
priority = 0

# Filter hooks - modify data in pipeline
[[hooks.filters]]
hook = "content_filter"
callback = "filter_content"
priority = 20

[[hooks.filters]]
hook = "post_title_filter"
callback = "filter_post_title"
priority = 0

# ============================================
# REST API ENDPOINTS
# ============================================
[api]
namespace = "my-plugin"            # API namespace (unique per plugin)
version = "v1"                     # API version

[[api.endpoints]]
path = "/data"                     # Endpoint path
method = "GET"                     # GET|POST|PUT|PATCH|DELETE
handler = "get_data"               # Handler function name
permission = "read"                # Required permission

[[api.endpoints]]
path = "/data"
method = "POST"
handler = "create_data"
permission = "edit_posts"
rate_limit = { requests = 100, window_seconds = 60 }

[[api.endpoints]]
path = "/data/:id"
method = "PUT"
handler = "update_data"
permission = "edit_posts"

[[api.endpoints]]
path = "/data/:id"
method = "DELETE"
handler = "delete_data"
permission = "delete_posts"

# ============================================
# DATABASE MIGRATIONS
# ============================================
[migrations]
directory = "migrations"           # Migrations folder
auto_run = true                    # Run on activation

[[migrations.files]]
version = "1.0.0"
file = "001_init.sql"

[[migrations.files]]
version = "1.1.0"
file = "002_add_index.sql"

# ============================================
# ASSETS (CSS/JS)
# ============================================
[[assets.css]]
path = "assets/style.css"
handle = "my-plugin-style"         # Unique handle
location = "header"                # header|footer

[[assets.css]]
path = "assets/admin.css"
handle = "my-plugin-admin-style"
location = "header"
admin_only = true                  # Only load in admin

[[assets.js]]
path = "assets/script.js"
handle = "my-plugin-script"
location = "footer"
dependencies = ["jquery", "alpine"]  # Load after these

[[assets.js]]
path = "assets/admin.js"
handle = "my-plugin-admin-script"
location = "footer"
admin_only = true

# ============================================
# ADMIN MENU & PAGES
# ============================================
[[admin.menu]]
id = "my-plugin"
label = "My Plugin"
icon = "settings"                  # Icon name
position = 20                      # Menu position (lower = higher)

[[admin.pages]]
id = "my-plugin-settings"
title = "Settings"
handler = "render_settings"
capability = "manage_options"      # Required capability

[[admin.pages]]
id = "my-plugin-dashboard"
title = "Dashboard"
handler = "render_dashboard"
capability = "read"

# ============================================
# SHORTCODES
# ============================================
[[shortcodes]]
tag = "my-shortcode"               # Usage: [my-shortcode]
handler = "render_shortcode"
supports_content = true            # Supports [tag]content[/tag]

[[shortcodes.attributes]]
name = "color"
attr_type = "string"
default = "blue"

[[shortcodes.attributes]]
name = "size"
attr_type = "integer"
default = 16

# ============================================
# GUTENBERG BLOCKS
# ============================================
[[blocks]]
name = "my-plugin/hero"            # namespace/block-name
title = "Hero Block"
category = "layout"                # layout|text|media|widgets|embed
render = "render_hero"

[blocks.attributes]
title = { attr_type = "string", default = "Hero Title" }
subtitle = { attr_type = "string", default = "" }
bg_color = { attr_type = "string", default = "#000000" }
text_color = { attr_type = "string", default = "#ffffff" }

[[blocks]]
name = "my-plugin/card"
title = "Card Block"
category = "layout"
render = "render_card"

[blocks.attributes]
heading = { attr_type = "string", default = "Card" }
content = { attr_type = "string", default = "" }

# ============================================
# WIDGETS
# ============================================
[[widgets]]
id = "my-widget"
name = "My Widget"
render = "render_widget"

[[widgets]]
id = "recent-items"
name = "Recent Items"
render = "render_recent_items"

# ============================================
# CLI COMMANDS
# ============================================
[[cli]]
name = "sync"
handler = "sync_handler"
description = "Sync data from external source"

[[cli.arguments]]
name = "source"
required = true

[[cli.options]]
name = "dry-run"
short = "d"

[[cli.options]]
name = "verbose"
short = "v"

[[cli]]
name = "export"
handler = "export_handler"
description = "Export plugin data"

[[cli.options]]
name = "format"
short = "f"

# ============================================
# CRON JOBS (Scheduled Tasks)
# ============================================
[[cron]]
name = "cleanup"
handler = "cleanup_handler"
schedule = "daily"                 # hourly|twice_daily|daily|weekly

[[cron]]
name = "sync"
handler = "sync_handler"
schedule = "0 */6 * * *"           # Cron expression (every 6 hours)

# ============================================
# MULTISITE SUPPORT
# ============================================
[network]
network_wide = true                # Activate network-wide
per_site = false                   # Or per-site settings

# ============================================
# WEBASSEMBLY (Sandboxed Execution)
# ============================================
[wasm]
memory_limit = 64                  # MB
timeout_ms = 5000                  # Max execution time

# ============================================
# FEATURE FLAGS
# ============================================
[features.beta_feature]
enabled = false
rollout_percentage = 10            # Gradual rollout

[features.new_ui]
enabled = true
rollout_percentage = 100
```

---

## DIRECTORY STRUCTURE

```
my-plugin/
├── plugin.toml              # REQUIRED: Plugin manifest
├── Cargo.toml               # REQUIRED: Rust dependencies
├── src/
│   ├── lib.rs               # REQUIRED: Main plugin struct & entry point
│   ├── config.rs            # Configuration handling
│   ├── api/                 # REST API handlers
│   │   ├── mod.rs
│   │   └── handlers.rs
│   ├── services/            # Business logic
│   │   ├── mod.rs
│   │   └── data_service.rs
│   ├── hooks/               # Hook implementations
│   │   ├── mod.rs
│   │   ├── actions.rs
│   │   └── filters.rs
│   └── models/              # Data structures
│       ├── mod.rs
│       └── entities.rs
├── migrations/              # SQL migrations
│   ├── 001_init.sql
│   └── 002_add_index.sql
├── templates/               # Admin page templates
│   ├── settings.html
│   └── dashboard.html
└── assets/                  # Static assets
    ├── style.css
    ├── admin.css
    ├── script.js
    └── admin.js
```

---

## RUST IMPLEMENTATION

### Main Plugin Struct (`src/lib.rs`)
```rust
use rustpress_plugins::prelude::*;

pub struct MyPlugin {
    info: PluginInfo,
    state: RwLock<PluginState>,
    config: RwLock<Option<MyConfig>>,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "my-plugin".into(),
                name: "My Plugin".into(),
                version: "1.0.0".into(),
            },
            state: RwLock::new(PluginState::Inactive),
            config: RwLock::new(None),
        }
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}
```

### Lifecycle Hooks Implementation
```rust
#[async_trait]
impl LifecycleHook for MyPlugin {
    /// Called when plugin is activated
    async fn on_activate(&self, ctx: &ActivationContext) -> Result<(), HookError> {
        // 1. Run database migrations
        ctx.run_migrations().await?;

        // 2. Initialize configuration
        let config = MyConfig::load(ctx.settings()).await?;
        *self.config.write().await = Some(config);

        // 3. Register routes/handlers
        ctx.register_routes(self.routes()).await?;

        // 4. Update state
        *self.state.write().await = PluginState::Active;

        Ok(())
    }

    /// Called when plugin is deactivated
    async fn on_deactivate(&self, ctx: &DeactivationContext) -> Result<(), HookError> {
        // 1. Cleanup resources (connections, caches, etc.)
        self.cleanup_resources().await?;

        // 2. Unregister routes
        ctx.unregister_routes().await?;

        // 3. Update state
        *self.state.write().await = PluginState::Inactive;

        Ok(())
    }

    /// Called when plugin version changes
    async fn on_upgrade(&self, ctx: &UpgradeContext) -> Result<(), HookError> {
        let from_version = &ctx.from_version;
        let to_version = &ctx.to_version;

        // Handle version-specific migrations
        if from_version < "1.1.0" && to_version >= "1.1.0" {
            self.migrate_to_1_1_0(ctx).await?;
        }

        Ok(())
    }

    /// Called when plugin is completely removed
    async fn on_uninstall(&self, ctx: &UninstallContext) -> Result<(), HookError> {
        // 1. Remove all plugin data from database
        ctx.db().execute("DROP TABLE IF EXISTS my_plugin_data").await?;

        // 2. Remove plugin settings
        ctx.settings().remove_all("my-plugin").await?;

        // 3. Clean up any files
        ctx.fs().remove_dir_all("my-plugin-uploads").await?;

        Ok(())
    }
}
```

### API Handlers
```rust
use axum::{Json, extract::{State, Path, Query}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ListParams {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Deserialize, Validate)]
pub struct CreateInput {
    #[validate(length(min = 1, max = 200))]
    name: String,
    #[validate(length(max = 1000))]
    description: Option<String>,
}

#[derive(Serialize)]
pub struct DataResponse {
    id: i64,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

/// GET /api/v1/my-plugin/data
pub async fn get_data(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<DataResponse>>, ApiError> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let data = sqlx::query_as!(
        DataResponse,
        r#"
        SELECT id, name, description, created_at
        FROM my_plugin_data
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user.id,
        per_page as i64,
        offset as i64
    )
    .fetch_all(state.db())
    .await?;

    Ok(Json(data))
}

/// POST /api/v1/my-plugin/data
pub async fn create_data(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    ValidatedJson(input): ValidatedJson<CreateInput>,
) -> Result<(StatusCode, Json<DataResponse>), ApiError> {
    let data = sqlx::query_as!(
        DataResponse,
        r#"
        INSERT INTO my_plugin_data (name, description, user_id)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, created_at
        "#,
        input.name,
        input.description,
        user.id
    )
    .fetch_one(state.db())
    .await?;

    Ok((StatusCode::CREATED, Json(data)))
}

/// PUT /api/v1/my-plugin/data/:id
pub async fn update_data(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(input): ValidatedJson<CreateInput>,
) -> Result<Json<DataResponse>, ApiError> {
    let data = sqlx::query_as!(
        DataResponse,
        r#"
        UPDATE my_plugin_data
        SET name = $1, description = $2
        WHERE id = $3 AND user_id = $4
        RETURNING id, name, description, created_at
        "#,
        input.name,
        input.description,
        id,
        user.id
    )
    .fetch_optional(state.db())
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(data))
}

/// DELETE /api/v1/my-plugin/data/:id
pub async fn delete_data(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query!(
        "DELETE FROM my_plugin_data WHERE id = $1 AND user_id = $2",
        id,
        user.id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
```

### Settings Access
```rust
impl MyPlugin {
    /// Get a typed setting value
    pub fn get_setting<T: FromSettingValue>(&self, key: &str) -> Option<T> {
        self.settings.get("my-plugin", key)?.try_into().ok()
    }

    /// Get setting with default fallback
    pub fn get_setting_or<T: FromSettingValue>(&self, key: &str, default: T) -> T {
        self.get_setting(key).unwrap_or(default)
    }

    /// Check if feature is enabled
    pub fn is_feature_enabled(&self) -> bool {
        self.get_setting_or("enable_feature", true)
    }

    /// Get API key (required setting)
    pub fn api_key(&self) -> Result<String, SettingError> {
        self.get_setting("api_key")
            .ok_or(SettingError::Required("api_key"))
    }
}
```

### Action Hook Handler
```rust
/// Called when a post is published
pub async fn on_post_published(
    ctx: &ActionContext,
    post: &Post,
) -> Result<(), HookError> {
    // Example: Send notification to external service
    let api_key = ctx.plugin().api_key()?;

    let client = reqwest::Client::new();
    client.post("https://api.example.com/notify")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "event": "post_published",
            "post_id": post.id,
            "title": post.title,
        }))
        .send()
        .await?;

    Ok(())
}
```

### Filter Hook Handler
```rust
/// Filter to modify post content before display
pub fn filter_content(
    ctx: &FilterContext,
    content: String,
) -> Result<String, HookError> {
    // Example: Add social share buttons
    let share_html = r#"
        <div class="share-buttons">
            <a href="#" class="share-twitter">Twitter</a>
            <a href="#" class="share-facebook">Facebook</a>
        </div>
    "#;

    Ok(format!("{}\n{}", content, share_html))
}
```

---

## DATABASE MIGRATION EXAMPLE

### `migrations/001_init.sql`
```sql
-- Create main data table
CREATE TABLE IF NOT EXISTS my_plugin_data (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_my_plugin_data_user_id ON my_plugin_data(user_id);
CREATE INDEX idx_my_plugin_data_status ON my_plugin_data(status);
CREATE INDEX idx_my_plugin_data_created_at ON my_plugin_data(created_at DESC);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_my_plugin_data_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER my_plugin_data_updated_at
    BEFORE UPDATE ON my_plugin_data
    FOR EACH ROW
    EXECUTE FUNCTION update_my_plugin_data_updated_at();
```

---

## SETTING TYPES REFERENCE

| Type | Description | Example Value |
|------|-------------|---------------|
| `string` | Single-line text | `"hello"` |
| `text` | Multi-line text | `"line1\nline2"` |
| `number` | Float number | `3.14` |
| `integer` | Whole number | `42` |
| `boolean` | True/false | `true` |
| `select` | Single choice dropdown | `"option1"` |
| `multiselect` | Multiple choice | `["opt1", "opt2"]` |
| `radio` | Radio buttons | `"choice"` |
| `checkbox` | Checkboxes | `["a", "b"]` |
| `color` | Color picker | `"#007bff"` |
| `date` | Date picker | `"2024-01-15"` |
| `datetime` | Date + time | `"2024-01-15T10:30:00Z"` |
| `file` | File upload | `"/uploads/file.pdf"` |
| `image` | Image upload | `"/uploads/logo.png"` |
| `url` | URL input | `"https://example.com"` |
| `email` | Email input | `"user@example.com"` |
| `password` | Masked input | `"secret"` |
| `json` | JSON editor | `{"key": "value"}` |
| `code` | Code editor | `"function() {}"` |

---

## HOOK PRIORITIES

| Constant | Value | Use Case |
|----------|-------|----------|
| `HIGHEST` | 100 | Run first (security checks) |
| `HIGH` | 50 | Before most handlers |
| `NORMAL` | 0 | Default priority |
| `LOW` | -50 | After most handlers |
| `LOWEST` | -100 | Run last (cleanup) |

---

## PLUGIN LIFECYCLE STATES

```
Discovered → Inactive → Activating → Active
                ↑                       ↓
                └── Deactivating ←──────┘
                         ↓
                    Uninstalling
```

---

## BEST PRACTICES

1. **Always define all capabilities in `plugin.toml`** - Don't dynamically add hooks/routes
2. **Implement ALL lifecycle hooks** - Even if empty, for future-proofing
3. **Use typed settings schemas** - Provides validation and admin UI
4. **Namespace APIs uniquely** - Use plugin ID as namespace
5. **Register asset dependencies** - Declare what your JS/CSS needs
6. **Create migrations for DB changes** - Never modify schema directly
7. **Use async for all I/O operations** - Don't block the event loop
8. **Handle errors with `Result`** - Never panic in plugin code
9. **Clean up on uninstall** - Remove all data, settings, and files
10. **Test lifecycle transitions** - Especially activate/deactivate cycles

---

## COMMON PERMISSIONS

| Permission | Description |
|------------|-------------|
| `read` | Can view content |
| `edit_posts` | Can create/edit posts |
| `delete_posts` | Can delete posts |
| `manage_options` | Can change settings |
| `manage_users` | Can manage users |
| `upload_files` | Can upload media |
| `administrator` | Full access |

---

## API ENDPOINT URL FORMAT

Endpoints are available at:
```
/api/{version}/{namespace}/{path}
```

Example for `namespace = "my-plugin"`, `version = "v1"`, `path = "/data"`:
```
GET  /api/v1/my-plugin/data
POST /api/v1/my-plugin/data
```
