# RustPress Advanced Hooks System

A comprehensive demonstration of RustPress hook patterns including actions, filters, shortcodes, caching, and event handling.

## Features

- **Action Hooks**: Side-effect handlers for system events
- **Filter Hooks**: Data transformation pipelines
- **Shortcodes**: Custom content rendering tags
- **Caching**: Option and query caching patterns
- **Event Bus**: Cross-system event communication
- **Utilities**: Common text processing functions

## Architecture

```
advanced-function/
├── function.toml       # Hook registrations
├── Cargo.toml          # Dependencies
└── src/
    └── lib.rs          # Implementation
        ├── actions     # Action hook handlers
        ├── filters     # Filter hook handlers
        ├── shortcodes  # Shortcode processors
        ├── cache       # Caching utilities
        ├── events      # Event bus system
        └── utils       # Helper functions
```

## Hook Patterns Demonstrated

### Action Hooks

Actions perform side effects in response to events:

```rust
// Handle post save
pub async fn on_post_save(ctx: ActionContext, data: ActionData) -> Result<(), HookError> {
    let post_id = data.get::<i64>()?;

    // Clear caches
    cache::invalidate_post(*post_id).await;

    // Emit event
    EVENT_BUS.emit("post_saved", json!({ "post_id": post_id })).await;

    Ok(())
}
```

**Registered Actions:**
- `init` - System initialization
- `save_post` - Post save events
- `delete_post` - Post deletion
- `user_register` - New user registration
- `user_login` - User login tracking
- `comment_post` - New comment processing
- `transition_post_status` - Status change handling

### Filter Hooks

Filters transform data through a pipeline:

```rust
// Process content
pub async fn process_content(ctx: FilterContext, content: String) -> Result<String, HookError> {
    let mut result = content;

    // Add table of contents
    result = add_table_of_contents(&result);

    // Auto-link URLs
    result = utils::auto_link_urls(&result);

    // Add lazy loading
    result = add_lazy_loading(&result);

    Ok(result)
}
```

**Registered Filters:**
- `the_content` - Post content processing
- `the_title` - Title processing
- `the_excerpt` - Excerpt generation
- `body_class` - Body CSS classes
- `post_class` - Post CSS classes
- `upload_mimes` - Allowed file types
- `sanitize_file_name` - File name cleaning
- `login_redirect` - Post-login redirection
- `rest_pre_dispatch` - API middleware

### Shortcodes

Shortcodes render custom content:

```rust
// Button shortcode
pub async fn button(ctx: ShortcodeContext, attrs: ShortcodeAttrs, _content: Option<String>) -> Result<String, HookError> {
    let text = attrs.get("text").unwrap_or("Button");
    let url = attrs.get("url").unwrap_or("#");
    let style = attrs.get("style").unwrap_or("primary");

    Ok(format!(r#"<a href="{}" class="btn btn-{}">{}</a>"#, url, style, text))
}
```

**Available Shortcodes:**

| Tag | Usage | Description |
|-----|-------|-------------|
| `[button]` | `[button text="Click" url="/page" style="primary"]` | Styled button |
| `[alert]` | `[alert type="warning"]Message[/alert]` | Alert box |
| `[accordion]` | `[accordion title="Title"]Content[/accordion]` | Collapsible section |
| `[tabs]` | `[tabs][tab title="Tab 1"]...[/tab][/tabs]` | Tabbed content |
| `[card]` | `[card title="Title" image="/img.jpg"]...[/card]` | Content card |
| `[recent_posts]` | `[recent_posts count="5" category="news"]` | Recent posts list |
| `[user_info]` | `[user_info field="name"]` | Current user data |
| `[site_stats]` | `[site_stats]` | Site statistics |
| `[code]` | `[code language="rust"]...[/code]` | Code block |
| `[embed]` | `[embed url="https://youtube.com/..."]` | Smart embed |

## Caching Pattern

```rust
// Cache warm-up on init
pub async fn warm_up(db: &Database) -> Result<(), HookError> {
    let options = ["site_name", "posts_per_page"];
    for name in options {
        if let Ok(Some(value)) = db.get_option(name).await {
            OPTION_CACHE.write().await.insert(name.to_string(), value);
        }
    }
    Ok(())
}

// Cache invalidation
pub async fn invalidate_post(post_id: i64) {
    // Clear post-specific caches
    // Clear listing caches
}
```

## Event Bus Pattern

```rust
// Emit events
EVENT_BUS.emit("post_published", json!({
    "post_id": post_id,
    "timestamp": Utc::now()
})).await;

// Subscribe to events
let mut rx = EVENT_BUS.subscribe("post_published").await;
while let Ok(event) = rx.recv().await {
    // Handle event
}
```

## Utility Functions

```rust
// Auto-link URLs in text
utils::auto_link_urls("Visit https://example.com");
// -> "Visit <a href=\"https://example.com\">https://example.com</a>"

// Smart quotes
utils::smart_quotes("He said \"hello\"");
// -> "He said "hello""

// Strip HTML
utils::strip_html("<p>Hello <b>world</b></p>");
// -> "Hello world"

// WordPress-style auto-paragraphs
utils::wpautop("Line 1\n\nLine 2");
// -> "<p>Line 1</p>\n<p>Line 2</p>"
```

## Best Practices

1. **Priority Ordering**: Use lower priorities (1-5) for critical hooks, 10 for standard
2. **Error Handling**: Always return proper `HookError` types
3. **Caching**: Cache expensive operations, invalidate on data changes
4. **Events**: Use event bus for decoupled cross-system communication
5. **Context Usage**: Leverage `ActionContext`/`FilterContext` for database and user access
6. **Async Safety**: Use `Arc<RwLock<>>` for shared mutable state

## License

MIT
