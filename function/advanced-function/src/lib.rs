//! RustPress Advanced Hooks System
//!
//! Demonstrates comprehensive hook implementations including:
//! - Action hooks for side effects
//! - Filter hooks for data transformation
//! - Shortcodes for content rendering
//! - Caching patterns
//! - Event handling
//! - Middleware patterns

use rustpress_functions::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================
// Module Structure
// ============================================

pub mod actions;
pub mod filters;
pub mod shortcodes;
pub mod cache;
pub mod events;
pub mod utils;

// ============================================
// Shared State
// ============================================

/// Global cache for option values
lazy_static::lazy_static! {
    static ref OPTION_CACHE: Arc<RwLock<HashMap<String, serde_json::Value>>> =
        Arc::new(RwLock::new(HashMap::new()));

    static ref EVENT_BUS: Arc<events::EventBus> =
        Arc::new(events::EventBus::new());
}

// ============================================
// Action Hooks
// ============================================

pub mod actions {
    use super::*;

    /// Initialize advanced features on system startup
    pub async fn on_init(ctx: ActionContext, _data: ActionData) -> Result<(), HookError> {
        tracing::info!("Advanced hooks system initialized");

        // Register custom post types, taxonomies, etc.
        register_custom_types(&ctx).await?;

        // Initialize event listeners
        setup_event_listeners().await;

        // Warm up caches
        cache::warm_up(&ctx.db).await?;

        Ok(())
    }

    /// Configure theme-related features
    pub async fn setup_theme_features(ctx: ActionContext, _data: ActionData) -> Result<(), HookError> {
        tracing::debug!("Setting up theme features");

        // Add theme support for various features
        // This would typically modify theme configuration

        Ok(())
    }

    /// Modify post queries globally
    pub async fn modify_query(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        // Access the query being built
        if let Some(query) = data.get::<PostQuery>() {
            tracing::debug!("Modifying query: {:?}", query);

            // Example: Exclude specific categories from main query
            // Example: Add custom ordering
        }

        Ok(())
    }

    /// Handle post save events
    pub async fn on_post_save(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        let post_id = data.get::<i64>().ok_or(HookError::InvalidData)?;

        tracing::info!("Post saved: {}", post_id);

        // Clear related caches
        cache::invalidate_post(*post_id).await;

        // Emit event for other systems
        EVENT_BUS.emit("post_saved", serde_json::json!({
            "post_id": post_id,
            "user_id": ctx.user.as_ref().map(|u| u.id),
            "timestamp": chrono::Utc::now()
        })).await;

        // Ping search engines for new content
        if let Some(post) = ctx.db.get_post(*post_id).await.ok().flatten() {
            if post.status == "published" {
                notify_search_engines(&post.url).await;
            }
        }

        Ok(())
    }

    /// Handle post deletion
    pub async fn on_post_delete(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        let post_id = data.get::<i64>().ok_or(HookError::InvalidData)?;

        tracing::info!("Post deleted: {}", post_id);

        // Clear all related caches
        cache::invalidate_post(*post_id).await;

        // Remove from search index
        // search::remove_from_index(*post_id).await;

        Ok(())
    }

    /// Welcome new users
    pub async fn on_user_register(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        let user_id = data.get::<i64>().ok_or(HookError::InvalidData)?;

        tracing::info!("New user registered: {}", user_id);

        // Send welcome email
        if let Some(user) = ctx.db.get_user(*user_id).await.ok().flatten() {
            send_welcome_email(&user).await?;
        }

        // Set default user preferences
        ctx.db.set_user_meta(*user_id, "notifications_enabled", "true").await.ok();

        // Add to newsletter (if opted in)
        // newsletter::subscribe(*user_id).await;

        Ok(())
    }

    /// Track user login activity
    pub async fn on_user_login(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        let user_id = data.get::<i64>().ok_or(HookError::InvalidData)?;

        // Update last login timestamp
        ctx.db.set_user_meta(*user_id, "last_login", &chrono::Utc::now().to_rfc3339()).await.ok();

        // Log login for security
        tracing::info!(
            user_id = %user_id,
            ip = %ctx.ip.as_deref().unwrap_or("unknown"),
            "User logged in"
        );

        Ok(())
    }

    /// Process new comments
    pub async fn on_new_comment(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        let comment_id = data.get::<i64>().ok_or(HookError::InvalidData)?;

        tracing::info!("New comment: {}", comment_id);

        // Check for spam (would use actual spam detection)
        // spam::check_comment(*comment_id).await;

        // Notify post author
        // notifications::notify_comment_author(*comment_id).await;

        // Clear post cache
        if let Some(comment) = ctx.db.get_comment(*comment_id).await.ok().flatten() {
            cache::invalidate_post(comment.post_id).await;
        }

        Ok(())
    }

    /// Handle post status transitions
    pub async fn on_status_change(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        #[derive(serde::Deserialize)]
        struct StatusChange {
            post_id: i64,
            old_status: String,
            new_status: String,
        }

        let change = data.get::<StatusChange>().ok_or(HookError::InvalidData)?;

        tracing::info!(
            "Post {} status changed: {} -> {}",
            change.post_id,
            change.old_status,
            change.new_status
        );

        // Handle specific transitions
        match (change.old_status.as_str(), change.new_status.as_str()) {
            ("draft", "published") => {
                // New publication - notify subscribers, social sharing
                EVENT_BUS.emit("post_published", serde_json::json!({
                    "post_id": change.post_id
                })).await;
            }
            ("published", "draft") => {
                // Unpublished - update caches
            }
            ("published", "trash") => {
                // Moved to trash - cleanup
            }
            _ => {}
        }

        Ok(())
    }

    /// Enhance email functionality
    pub async fn on_email_send(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
        // Add custom headers, tracking pixels, etc.
        tracing::debug!("Email being sent");
        Ok(())
    }

    // Helper functions

    async fn register_custom_types(ctx: &ActionContext) -> Result<(), HookError> {
        // Register custom post types, taxonomies, etc.
        Ok(())
    }

    async fn setup_event_listeners() {
        // Set up listeners for cross-system events
    }

    async fn notify_search_engines(url: &str) {
        // Ping Google, Bing, etc.
        tracing::debug!("Notifying search engines about: {}", url);
    }

    async fn send_welcome_email(user: &User) -> Result<(), HookError> {
        tracing::info!("Sending welcome email to: {}", user.email);
        Ok(())
    }
}

// ============================================
// Filter Hooks
// ============================================

pub mod filters {
    use super::*;

    /// Process and enhance post content
    pub async fn process_content(ctx: FilterContext, content: String) -> Result<String, HookError> {
        let mut result = content;

        // Add table of contents for long posts
        if result.matches("<h2").count() >= 3 {
            result = add_table_of_contents(&result);
        }

        // Auto-link URLs
        result = utils::auto_link_urls(&result);

        // Add lazy loading to images
        result = add_lazy_loading(&result);

        // Add anchor links to headings
        result = add_heading_anchors(&result);

        // Process shortcodes (if not already done)
        result = process_shortcodes(&result).await;

        Ok(result)
    }

    /// Process post titles
    pub async fn process_title(ctx: FilterContext, title: String) -> Result<String, HookError> {
        // Decode HTML entities
        let result = html_escape::decode_html_entities(&title).to_string();

        // Smart quotes
        let result = utils::smart_quotes(&result);

        Ok(result)
    }

    /// Generate smart excerpts
    pub async fn auto_excerpt(ctx: FilterContext, excerpt: String) -> Result<String, HookError> {
        if !excerpt.is_empty() {
            return Ok(excerpt);
        }

        // If no excerpt, generate from content
        if let Some(post) = &ctx.post {
            let plain_text = utils::strip_html(&post.content);
            let words: Vec<&str> = plain_text.split_whitespace().take(55).collect();
            return Ok(words.join(" ") + "...");
        }

        Ok(excerpt)
    }

    /// Add custom body classes
    pub async fn add_body_classes(ctx: FilterContext, classes: Vec<String>) -> Result<Vec<String>, HookError> {
        let mut result = classes;

        // Add user-specific classes
        if let Some(user) = &ctx.user {
            result.push("logged-in".to_string());
            if user.is_admin() {
                result.push("admin-bar".to_string());
            }
        } else {
            result.push("logged-out".to_string());
        }

        // Add device class
        if let Some(ua) = &ctx.user_agent {
            if ua.contains("Mobile") {
                result.push("mobile".to_string());
            } else {
                result.push("desktop".to_string());
            }
        }

        // Add theme class
        if let Some(theme) = ctx.get_option("color_scheme").await {
            result.push(format!("theme-{}", theme));
        }

        Ok(result)
    }

    /// Add custom post classes
    pub async fn add_post_classes(ctx: FilterContext, classes: Vec<String>) -> Result<Vec<String>, HookError> {
        let mut result = classes;

        if let Some(post) = &ctx.post {
            // Add category classes
            for cat in &post.categories {
                result.push(format!("category-{}", cat.slug));
            }

            // Add tag classes
            for tag in &post.tags {
                result.push(format!("tag-{}", tag.slug));
            }

            // Add format class
            if let Some(format) = &post.format {
                result.push(format!("format-{}", format));
            }

            // Has featured image
            if post.featured_image.is_some() {
                result.push("has-post-thumbnail".to_string());
            }
        }

        Ok(result)
    }

    /// Customize menu items
    pub async fn menu_item_args(ctx: FilterContext, args: MenuItemArgs) -> Result<MenuItemArgs, HookError> {
        let mut result = args;

        // Add icons based on title or URL
        if result.title.to_lowercase().contains("home") {
            result.before = Some("<span class=\"icon icon-home\"></span>".to_string());
        }

        // Mark external links
        if let Some(url) = &result.url {
            if !url.starts_with('/') && !url.contains(&ctx.site_url) {
                result.classes.push("external-link".to_string());
                result.after = Some("<span class=\"icon icon-external\"></span>".to_string());
            }
        }

        Ok(result)
    }

    /// Allow additional file types for upload
    pub async fn extend_mime_types(ctx: FilterContext, mimes: HashMap<String, String>) -> Result<HashMap<String, String>, HookError> {
        let mut result = mimes;

        // Add SVG support (with caution)
        result.insert("svg".to_string(), "image/svg+xml".to_string());

        // Add WebP
        result.insert("webp".to_string(), "image/webp".to_string());

        // Add font types
        result.insert("woff".to_string(), "font/woff".to_string());
        result.insert("woff2".to_string(), "font/woff2".to_string());

        Ok(result)
    }

    /// Sanitize uploaded file names
    pub async fn clean_filename(ctx: FilterContext, filename: String) -> Result<String, HookError> {
        let mut result = filename;

        // Remove special characters
        result = result.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");

        // Convert to lowercase
        result = result.to_lowercase();

        // Replace spaces with dashes
        result = result.replace(' ', "-");

        // Remove multiple dashes
        let re = regex::Regex::new(r"-+").unwrap();
        result = re.replace_all(&result, "-").to_string();

        // Trim dashes from ends
        result = result.trim_matches('-').to_string();

        Ok(result)
    }

    /// Cache option lookups
    pub async fn cache_option(ctx: FilterContext, value: Option<String>) -> Result<Option<String>, HookError> {
        // Check cache first
        let option_name = ctx.filter_args.get("option_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !option_name.is_empty() {
            let cache = OPTION_CACHE.read().await;
            if let Some(cached) = cache.get(option_name) {
                return Ok(Some(cached.to_string()));
            }
        }

        // Return original value (will be cached by post_option filter)
        Ok(value)
    }

    /// Process text widget content
    pub async fn process_widget_text(ctx: FilterContext, text: String) -> Result<String, HookError> {
        // Apply content filters to widget text
        let mut result = text;

        // Process shortcodes
        result = process_shortcodes(&result).await;

        // Auto paragraphs
        result = utils::wpautop(&result);

        Ok(result)
    }

    /// Process and sanitize comments
    pub async fn process_comment(ctx: FilterContext, text: String) -> Result<String, HookError> {
        let mut result = text;

        // Escape HTML
        result = html_escape::encode_text(&result).to_string();

        // Convert newlines to <br>
        result = result.replace("\n", "<br>");

        // Auto-link URLs (safely)
        result = utils::auto_link_urls(&result);

        Ok(result)
    }

    /// Custom post-login redirection
    pub async fn custom_login_redirect(ctx: FilterContext, redirect_to: String) -> Result<String, HookError> {
        if let Some(user) = &ctx.user {
            // Admin users go to dashboard
            if user.is_admin() {
                return Ok("/admin".to_string());
            }

            // Authors go to their profile
            if user.role == "author" {
                return Ok("/profile".to_string());
            }
        }

        // Default redirect
        Ok(redirect_to)
    }

    /// API request middleware
    pub async fn api_middleware(ctx: FilterContext, result: Option<Response>) -> Result<Option<Response>, HookError> {
        // Rate limiting check
        if let Some(ip) = &ctx.ip {
            if is_rate_limited(ip).await {
                return Ok(Some(Response::error(429, "Too many requests")));
            }
        }

        // API key validation for certain endpoints
        // Authentication checks
        // Request logging

        Ok(result)
    }

    /// Register custom query variables
    pub async fn add_query_vars(ctx: FilterContext, vars: Vec<String>) -> Result<Vec<String>, HookError> {
        let mut result = vars;
        result.push("custom_filter".to_string());
        result.push("sort_by".to_string());
        result.push("date_range".to_string());
        Ok(result)
    }

    /// Modify incoming requests
    pub async fn modify_request(ctx: FilterContext, request: Request) -> Result<Request, HookError> {
        // Add custom processing based on request parameters
        Ok(request)
    }

    // Helper functions

    fn add_table_of_contents(content: &str) -> String {
        // Parse headings and generate TOC
        content.to_string()
    }

    fn add_lazy_loading(content: &str) -> String {
        content.replace("<img ", "<img loading=\"lazy\" ")
    }

    fn add_heading_anchors(content: &str) -> String {
        // Add anchor links to h2, h3, etc.
        content.to_string()
    }

    async fn process_shortcodes(content: &str) -> String {
        // Process [shortcode] tags
        content.to_string()
    }

    async fn is_rate_limited(ip: &str) -> bool {
        // Check rate limit status
        false
    }
}

// ============================================
// Shortcodes
// ============================================

pub mod shortcodes {
    use super::*;

    /// Styled button shortcode
    /// Usage: [button text="Click Me" url="/page" style="primary" size="large"]
    pub async fn button(ctx: ShortcodeContext, attrs: ShortcodeAttrs, _content: Option<String>) -> Result<String, HookError> {
        let text = attrs.get("text").unwrap_or("Button");
        let url = attrs.get("url").unwrap_or("#");
        let style = attrs.get("style").unwrap_or("primary");
        let size = attrs.get("size").unwrap_or("medium");
        let target = if attrs.get("new_tab").map(|v| v == "true").unwrap_or(false) {
            " target=\"_blank\" rel=\"noopener\""
        } else {
            ""
        };

        Ok(format!(
            r#"<a href="{}" class="btn btn-{} btn-{}"{}">{}</a>"#,
            url, style, size, target, text
        ))
    }

    /// Alert box shortcode
    /// Usage: [alert type="warning"]Your message here[/alert]
    pub async fn alert(ctx: ShortcodeContext, attrs: ShortcodeAttrs, content: Option<String>) -> Result<String, HookError> {
        let alert_type = attrs.get("type").unwrap_or("info");
        let title = attrs.get("title");
        let content = content.unwrap_or_default();

        let icons = HashMap::from([
            ("info", "ℹ️"),
            ("success", "✅"),
            ("warning", "⚠️"),
            ("error", "❌"),
        ]);

        let icon = icons.get(alert_type).unwrap_or(&"ℹ️");

        let title_html = title.map(|t| format!("<strong class=\"alert-title\">{}</strong>", t)).unwrap_or_default();

        Ok(format!(
            r#"<div class="alert alert-{}" role="alert">
                <span class="alert-icon">{}</span>
                <div class="alert-content">{}{}</div>
            </div>"#,
            alert_type, icon, title_html, content
        ))
    }

    /// Accordion shortcode
    /// Usage: [accordion title="Section Title"]Content here[/accordion]
    pub async fn accordion(ctx: ShortcodeContext, attrs: ShortcodeAttrs, content: Option<String>) -> Result<String, HookError> {
        let title = attrs.get("title").unwrap_or("Accordion");
        let open = attrs.get("open").map(|v| v == "true").unwrap_or(false);
        let content = content.unwrap_or_default();
        let id = uuid::Uuid::new_v4();

        Ok(format!(
            r#"<div class="accordion">
                <button class="accordion-header" aria-expanded="{}" aria-controls="accordion-{}">
                    <span>{}</span>
                    <span class="accordion-icon"></span>
                </button>
                <div id="accordion-{}" class="accordion-content" {}>
                    {}
                </div>
            </div>"#,
            open, id, title, id,
            if open { "" } else { "hidden" },
            content
        ))
    }

    /// Tabs shortcode
    /// Usage: [tabs][tab title="Tab 1"]Content 1[/tab][tab title="Tab 2"]Content 2[/tab][/tabs]
    pub async fn tabs(ctx: ShortcodeContext, attrs: ShortcodeAttrs, content: Option<String>) -> Result<String, HookError> {
        // Parse nested [tab] shortcodes
        let content = content.unwrap_or_default();
        let id = uuid::Uuid::new_v4();

        // Simplified - in reality would parse nested shortcodes
        Ok(format!(
            r#"<div class="tabs" id="tabs-{}">
                <div class="tab-list" role="tablist">
                    <!-- Tab buttons would be generated here -->
                </div>
                <div class="tab-panels">
                    {}
                </div>
            </div>"#,
            id, content
        ))
    }

    /// Card shortcode
    /// Usage: [card title="Title" image="/path/to/image.jpg"]Description[/card]
    pub async fn card(ctx: ShortcodeContext, attrs: ShortcodeAttrs, content: Option<String>) -> Result<String, HookError> {
        let title = attrs.get("title");
        let image = attrs.get("image");
        let link = attrs.get("link");
        let content = content.unwrap_or_default();

        let image_html = image.map(|img| format!(
            r#"<div class="card-image"><img src="{}" alt="" loading="lazy"></div>"#,
            img
        )).unwrap_or_default();

        let title_html = title.map(|t| format!("<h3 class=\"card-title\">{}</h3>", t)).unwrap_or_default();

        let wrapper_start = link.map(|l| format!(r#"<a href="{}" class="card-link">"#, l)).unwrap_or_default();
        let wrapper_end = if link.is_some() { "</a>" } else { "" };

        Ok(format!(
            r#"<div class="card">
                {}
                {}
                <div class="card-body">
                    {}
                    <div class="card-content">{}</div>
                </div>
                {}
            </div>"#,
            wrapper_start, image_html, title_html, content, wrapper_end
        ))
    }

    /// Recent posts shortcode
    /// Usage: [recent_posts count="5" category="news"]
    pub async fn recent_posts(ctx: ShortcodeContext, attrs: ShortcodeAttrs, _content: Option<String>) -> Result<String, HookError> {
        let count: i64 = attrs.get("count").and_then(|v| v.parse().ok()).unwrap_or(5);
        let category = attrs.get("category");

        // This would query the database
        // let posts = ctx.db.get_recent_posts(count, category).await?;

        // Placeholder
        Ok(format!(
            r#"<div class="recent-posts">
                <ul>
                    <li><a href="#">Recent Post 1</a></li>
                    <li><a href="#">Recent Post 2</a></li>
                    <li><a href="#">Recent Post 3</a></li>
                </ul>
            </div>"#
        ))
    }

    /// User info shortcode
    /// Usage: [user_info field="name"]
    pub async fn user_info(ctx: ShortcodeContext, attrs: ShortcodeAttrs, _content: Option<String>) -> Result<String, HookError> {
        let field = attrs.get("field").unwrap_or("name");

        if let Some(user) = &ctx.user {
            let value = match field {
                "name" => &user.name,
                "email" => &user.email,
                "role" => &user.role,
                _ => "",
            };
            Ok(value.to_string())
        } else {
            Ok(attrs.get("default").unwrap_or("Guest").to_string())
        }
    }

    /// Site statistics shortcode
    /// Usage: [site_stats]
    pub async fn site_stats(ctx: ShortcodeContext, attrs: ShortcodeAttrs, _content: Option<String>) -> Result<String, HookError> {
        // This would query actual statistics
        Ok(format!(
            r#"<div class="site-stats">
                <div class="stat">
                    <span class="stat-value">150</span>
                    <span class="stat-label">Posts</span>
                </div>
                <div class="stat">
                    <span class="stat-value">1.2K</span>
                    <span class="stat-label">Comments</span>
                </div>
                <div class="stat">
                    <span class="stat-value">50K</span>
                    <span class="stat-label">Views</span>
                </div>
            </div>"#
        ))
    }

    /// Code block shortcode with syntax highlighting
    /// Usage: [code language="rust"]fn main() {}[/code]
    pub async fn code_block(ctx: ShortcodeContext, attrs: ShortcodeAttrs, content: Option<String>) -> Result<String, HookError> {
        let language = attrs.get("language").unwrap_or("text");
        let content = content.unwrap_or_default();
        let escaped = html_escape::encode_text(&content);

        Ok(format!(
            r#"<pre><code class="language-{}">{}</code></pre>"#,
            language, escaped
        ))
    }

    /// Smart embed shortcode
    /// Usage: [embed url="https://youtube.com/watch?v=..."]
    pub async fn smart_embed(ctx: ShortcodeContext, attrs: ShortcodeAttrs, _content: Option<String>) -> Result<String, HookError> {
        let url_str = attrs.get("url").ok_or(HookError::InvalidData)?;
        let width = attrs.get("width").unwrap_or("560");
        let height = attrs.get("height").unwrap_or("315");

        let url = url::Url::parse(url_str).map_err(|_| HookError::InvalidData)?;

        // Detect platform and generate appropriate embed
        let host = url.host_str().unwrap_or("");

        let embed_html = if host.contains("youtube.com") || host.contains("youtu.be") {
            let video_id = extract_youtube_id(&url);
            format!(
                r#"<div class="embed embed-youtube">
                    <iframe width="{}" height="{}" src="https://www.youtube.com/embed/{}"
                        frameborder="0" allowfullscreen loading="lazy"></iframe>
                </div>"#,
                width, height, video_id.unwrap_or("")
            )
        } else if host.contains("vimeo.com") {
            format!(
                r#"<div class="embed embed-vimeo">
                    <iframe width="{}" height="{}" src="{}"
                        frameborder="0" allowfullscreen loading="lazy"></iframe>
                </div>"#,
                width, height, url_str
            )
        } else if host.contains("twitter.com") || host.contains("x.com") {
            format!(
                r#"<div class="embed embed-twitter">
                    <blockquote class="twitter-tweet"><a href="{}"></a></blockquote>
                </div>"#,
                url_str
            )
        } else {
            // Generic embed/link
            format!(r#"<a href="{}" class="external-link">{}</a>"#, url_str, url_str)
        };

        Ok(embed_html)
    }

    fn extract_youtube_id(url: &url::Url) -> Option<&str> {
        url.query_pairs()
            .find(|(k, _)| k == "v")
            .map(|(_, v)| v)
            .map(|v| v.into_owned())
            .or_else(|| {
                url.path_segments()?.last().map(String::from)
            })
            .map(|s| Box::leak(s.into_boxed_str()) as &str)
    }
}

// ============================================
// Cache Module
// ============================================

pub mod cache {
    use super::*;

    /// Warm up caches on initialization
    pub async fn warm_up(db: &Database) -> Result<(), HookError> {
        tracing::info!("Warming up caches");

        // Cache frequently accessed options
        let options = ["site_name", "site_description", "admin_email", "posts_per_page"];
        for name in options {
            if let Ok(Some(value)) = db.get_option(name).await {
                let mut cache = OPTION_CACHE.write().await;
                cache.insert(name.to_string(), serde_json::json!(value));
            }
        }

        Ok(())
    }

    /// Invalidate caches related to a post
    pub async fn invalidate_post(post_id: i64) {
        tracing::debug!("Invalidating cache for post: {}", post_id);
        // Clear post-specific cache keys
        // Clear listing caches
        // Clear related sidebar widgets
    }

    /// Clear all caches
    pub async fn clear_all() {
        let mut cache = OPTION_CACHE.write().await;
        cache.clear();
        tracing::info!("All caches cleared");
    }
}

// ============================================
// Events Module
// ============================================

pub mod events {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::broadcast;

    pub struct EventBus {
        channels: RwLock<HashMap<String, broadcast::Sender<serde_json::Value>>>,
    }

    impl EventBus {
        pub fn new() -> Self {
            Self {
                channels: RwLock::new(HashMap::new()),
            }
        }

        pub async fn emit(&self, event: &str, data: serde_json::Value) {
            let channels = self.channels.read().await;
            if let Some(tx) = channels.get(event) {
                let _ = tx.send(data);
            }
            tracing::debug!("Event emitted: {}", event);
        }

        pub async fn subscribe(&self, event: &str) -> broadcast::Receiver<serde_json::Value> {
            let mut channels = self.channels.write().await;
            let tx = channels
                .entry(event.to_string())
                .or_insert_with(|| broadcast::channel(100).0);
            tx.subscribe()
        }
    }
}

// ============================================
// Utilities Module
// ============================================

pub mod utils {
    /// Convert plain text URLs to clickable links
    pub fn auto_link_urls(text: &str) -> String {
        let url_regex = regex::Regex::new(
            r"(https?://[^\s<>\[\]]+)"
        ).unwrap();

        url_regex.replace_all(text, r#"<a href="$1">$1</a>"#).to_string()
    }

    /// Convert straight quotes to smart quotes
    pub fn smart_quotes(text: &str) -> String {
        text.replace("\"", """)
            .replace("'", "'")
    }

    /// Strip HTML tags from text
    pub fn strip_html(html: &str) -> String {
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        re.replace_all(html, "").to_string()
    }

    /// WordPress-style auto paragraphs
    pub fn wpautop(text: &str) -> String {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        paragraphs.iter()
            .map(|p| format!("<p>{}</p>", p.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
