# Sample Hooks & Functions

A minimal RustPress hooks example demonstrating the event system.

## Features

- **Hook Registry**: Central registration and execution
- **Action Hooks**: Event-based callbacks (no return value)
- **Filter Hooks**: Data transformation pipeline
- **Lifecycle Hooks**: Component activation/deactivation
- **Priority System**: Control execution order

## Hook Types

### Action Hooks
Fire-and-forget events:
```rust
registry.add_action("post_publish", on_post_published, priority::NORMAL).await;
registry.do_action("post_publish", &ctx, post_id).await?;
```

### Filter Hooks
Transform data through a pipeline:
```rust
registry.add_filter("content", filter_add_nofollow, priority::HIGH).await;
let result = registry.apply_filters("content", &ctx, content).await?;
```

### Lifecycle Hooks
Component state management:
```rust
impl LifecycleHook for MyComponent {
    async fn on_activate(&self) -> Result<(), HookError>;
    async fn on_deactivate(&self) -> Result<(), HookError>;
}
```

## Priorities

| Constant | Value | Use Case |
|----------|-------|----------|
| `HIGHEST` | 100 | Run first |
| `HIGH` | 50 | Before most |
| `NORMAL` | 0 | Default |
| `LOW` | -50 | After most |
| `LOWEST` | -100 | Run last |

## File Structure

```
sample-function/
├── Cargo.toml
└── src/
    └── lib.rs    # Registry, handlers, lifecycle
```

## Running Tests

```bash
cargo test
```
