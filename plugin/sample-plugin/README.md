# Hello World Plugin

A minimal RustPress plugin example demonstrating core concepts.

## Features

- **Settings**: Configurable greeting message
- **REST API**: GET/POST endpoints for greeting
- **Shortcode**: `[hello name="User"]` renders a greeting
- **Lifecycle**: Proper activate/deactivate handling

## Usage

### Shortcode

```html
[hello]                    <!-- Outputs: Hello, World! -->
[hello name="RustPress"]   <!-- Outputs: Hello, RustPress! -->
```

### API

```bash
# Get greeting
GET /api/v1/hello-world/greet

# Set greeting (requires edit_posts permission)
POST /api/v1/hello-world/greet
{"message": "Welcome!"}
```

## File Structure

```
hello-world/
├── plugin.toml      # Plugin manifest
├── Cargo.toml       # Rust dependencies
├── src/
│   └── lib.rs       # Main implementation
└── migrations/
    └── 001_init.sql # Database setup (optional)
```
