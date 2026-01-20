# RustPress Blog API

A comprehensive blog API demonstrating advanced RustPress app development patterns.

## Features

- **Posts**: Full CRUD operations with drafts, scheduling, and publishing workflow
- **Categories**: Hierarchical category system with nested support
- **Tags**: Flexible tagging system
- **Comments**: Threaded comments with moderation support
- **Media**: File upload and management
- **Search**: Full-text search using PostgreSQL
- **RSS Feed**: Auto-generated RSS feed
- **Caching**: Response caching with Redis
- **Rate Limiting**: Per-client request limiting

## Architecture

```
advanced-app/
├── app.toml              # App manifest with routes, middleware, permissions
├── Cargo.toml            # Rust dependencies
├── migrations/           # Database migrations
│   └── 001_init.sql      # Initial schema
└── src/
    ├── lib.rs            # App entry point and router setup
    ├── models.rs         # Data models and DTOs
    ├── services.rs       # Business logic services
    ├── handlers/         # HTTP request handlers
    │   ├── mod.rs
    │   ├── posts.rs      # Post endpoints
    │   ├── comments.rs   # Comment endpoints
    │   ├── categories.rs # Category endpoints
    │   ├── tags.rs       # Tag endpoints
    │   ├── media.rs      # Media upload endpoints
    │   ├── search.rs     # Search endpoint
    │   ├── feed.rs       # RSS feed
    │   └── admin.rs      # Admin endpoints
    ├── middleware/       # Custom middleware
    │   ├── mod.rs
    │   ├── auth.rs       # Authentication
    │   ├── cache.rs      # Response caching
    │   ├── rate_limit.rs # Rate limiting
    │   └── view_counter.rs
    └── extractors/       # Custom Axum extractors
        └── mod.rs        # AuthUser, ClientInfo, Pagination
```

## Key Patterns Demonstrated

### 1. Route Organization
- Public routes (no auth): List posts, view post, search
- Protected routes (auth required): Create/edit posts, upload media
- Admin routes: Statistics, moderation

### 2. Service Layer
- Separation of HTTP handling from business logic
- Caching integration
- Error handling with custom error types

### 3. Custom Extractors
- `AuthUser`: Extract and validate authenticated user
- `ClientInfo`: Extract IP and user agent
- `Pagination`: Parse pagination query params

### 4. Middleware Stack
- Authentication validation
- Response caching
- Rate limiting
- Request logging

### 5. Input Validation
- Using `validator` crate for request validation
- Custom validation rules per model

## API Endpoints

### Public

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/posts` | List published posts |
| GET | `/posts/:slug` | Get post by slug |
| GET | `/posts/:id/comments` | List post comments |
| POST | `/posts/:id/comments` | Create comment |
| GET | `/categories` | List categories |
| GET | `/tags` | List tags |
| GET | `/search?q=term` | Search posts |
| GET | `/feed` | RSS feed |

### Protected (Requires Auth)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/posts` | Create post |
| PUT | `/posts/:id` | Update post |
| DELETE | `/posts/:id` | Delete post |
| POST | `/posts/:id/publish` | Publish post |
| POST | `/posts/:id/unpublish` | Unpublish post |
| GET | `/drafts` | List user's drafts |
| GET | `/media` | List user's media |
| POST | `/media` | Upload media file |
| DELETE | `/media/:id` | Delete media |

### Admin

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/admin/posts` | All posts |
| GET | `/admin/comments/pending` | Pending comments |
| GET | `/admin/stats` | Blog statistics |

## Query Parameters

### Posts List
- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 10, max: 100)
- `category`: Filter by category slug
- `tag`: Filter by tag slug
- `sort`: Sort field (date, views, comments)
- `order`: Sort order (asc, desc)

### Search
- `q`: Search query (min 3 chars)
- `page`, `per_page`: Pagination
- `category`, `tag`: Optional filters

## Error Responses

All errors return JSON:

```json
{
  "error": "error_code",
  "message": "Human readable message",
  "details": { ... }
}
```

Error codes: `not_found`, `validation_error`, `unauthorized`, `forbidden`, `rate_limited`

## License

MIT
