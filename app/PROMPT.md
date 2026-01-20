# RustPress App/Routing Development - AI Prompt Reference

You are building routing, handlers, and services for RustPress, a Rust-based CMS using Axum. Follow this specification exactly.

---

## CORE APPLICATION STATE

### AppState Structure
```rust
use sqlx::PgPool;
use deadpool_redis::Pool as RedisPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,                          // PostgreSQL connection pool
    pub redis: RedisPool,                    // Redis connection pool
    pub config: Arc<AppConfig>,              // Application configuration
    pub plugins: Arc<PluginRegistry>,        // Plugin management
    pub themes: Arc<ThemeManager>,           // Theme management
    pub hooks: Arc<HookRegistry>,            // Hook/Function registry
}

impl AppState {
    /// Get database pool reference
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    /// Get Redis pool reference
    pub fn redis(&self) -> &RedisPool {
        &self.redis
    }

    /// Get configuration reference
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}
```

---

## ROUTER SETUP

### Basic Router Configuration
```rust
use axum::{Router, routing::{get, post, put, patch, delete}};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // API routes (versioned)
        .nest("/api/v1", api_v1_routes())
        // Admin dashboard routes
        .nest("/admin", admin_routes())
        // Public/frontend routes
        .merge(public_routes())
        // Attach state to all routes
        .with_state(state)
}
```

### Route Group Organization
```rust
/// API v1 routes
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/posts", posts_routes())
        .nest("/users", users_routes())
        .nest("/auth", auth_routes())
        .nest("/media", media_routes())
        .nest("/settings", settings_routes())
}

/// Posts CRUD routes
fn posts_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/:id", get(get_post).put(update_post).delete(delete_post))
        .route("/:id/publish", post(publish_post))
        .route("/:id/comments", get(list_comments).post(create_comment))
}

/// Users routes
fn users_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/me", get(get_current_user).put(update_current_user))
}

/// Authentication routes
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/register", post(register))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

/// Admin routes
fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(admin_dashboard))
        .route("/posts", get(admin_posts))
        .route("/users", get(admin_users))
        .route("/plugins", get(admin_plugins))
        .route("/settings", get(admin_settings).post(save_settings))
        // Apply auth middleware to all admin routes
        .layer(middleware::from_fn_with_state(state, require_admin))
}

/// Public routes
fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/page/:slug", get(page))
        .route("/post/:slug", get(single_post))
        .route("/category/:slug", get(category_archive))
        .route("/author/:slug", get(author_archive))
        .route("/search", get(search))
}
```

---

## HANDLER PATTERNS

### Basic Handler (No Parameters)
```rust
pub async fn list_posts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, ApiError> {
    let posts = sqlx::query_as!(Post, "SELECT * FROM posts WHERE status = 'published'")
        .fetch_all(state.db())
        .await?;
    Ok(Json(posts))
}
```

### With Path Parameters
```rust
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Post>, ApiError> {
    let post = sqlx::query_as!(Post, "SELECT * FROM posts WHERE id = $1", id)
        .fetch_optional(state.db())
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(post))
}

// Multiple path parameters
pub async fn get_comment(
    State(state): State<AppState>,
    Path((post_id, comment_id)): Path<(i64, i64)>,
) -> Result<Json<Comment>, ApiError> {
    let comment = sqlx::query_as!(
        Comment,
        "SELECT * FROM comments WHERE id = $1 AND post_id = $2",
        comment_id,
        post_id
    )
    .fetch_optional(state.db())
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(comment))
}
```

### With Query Parameters
```rust
#[derive(Deserialize)]
pub struct ListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    sort: Option<String>,        // "created_at" | "title" | "updated_at"
    order: Option<String>,       // "asc" | "desc"
    status: Option<String>,      // "draft" | "published" | "archived"
    search: Option<String>,
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<Post>>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT * FROM posts
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR title ILIKE '%' || $2 || '%')
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        params.status,
        params.search,
        per_page as i64,
        offset as i64
    )
    .fetch_all(state.db())
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM posts
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR title ILIKE '%' || $2 || '%')
        "#,
        params.status,
        params.search
    )
    .fetch_one(state.db())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: posts,
        pagination: Pagination {
            page,
            per_page,
            total: total as u32,
            total_pages: ((total as f64) / (per_page as f64)).ceil() as u32,
        },
    }))
}
```

### With JSON Body (Create)
```rust
#[derive(Deserialize, Validate)]
pub struct CreatePostInput {
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    title: String,

    #[validate(length(min = 1, message = "Content is required"))]
    content: String,

    #[validate(length(max = 500))]
    excerpt: Option<String>,

    status: Option<String>,
    category_ids: Option<Vec<i64>>,
    tag_ids: Option<Vec<i64>>,
}

pub async fn create_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    ValidatedJson(input): ValidatedJson<CreatePostInput>,
) -> Result<(StatusCode, Json<Post>), ApiError> {
    // Generate slug from title
    let slug = slugify(&input.title);
    let status = input.status.unwrap_or_else(|| "draft".to_string());

    let post = sqlx::query_as!(
        Post,
        r#"
        INSERT INTO posts (title, slug, content, excerpt, status, author_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        input.title,
        slug,
        input.content,
        input.excerpt,
        status,
        user.id
    )
    .fetch_one(state.db())
    .await?;

    // Handle categories if provided
    if let Some(category_ids) = input.category_ids {
        for cat_id in category_ids {
            sqlx::query!(
                "INSERT INTO post_categories (post_id, category_id) VALUES ($1, $2)",
                post.id,
                cat_id
            )
            .execute(state.db())
            .await?;
        }
    }

    Ok((StatusCode::CREATED, Json(post)))
}
```

### With JSON Body (Update)
```rust
#[derive(Deserialize, Validate)]
pub struct UpdatePostInput {
    #[validate(length(min = 1, max = 200))]
    title: Option<String>,
    content: Option<String>,
    excerpt: Option<String>,
    status: Option<String>,
}

pub async fn update_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(input): ValidatedJson<UpdatePostInput>,
) -> Result<Json<Post>, ApiError> {
    // Check ownership or admin
    let existing = sqlx::query!("SELECT author_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db())
        .await?
        .ok_or(ApiError::NotFound)?;

    if existing.author_id != user.id && !user.is_admin() {
        return Err(ApiError::Forbidden);
    }

    let post = sqlx::query_as!(
        Post,
        r#"
        UPDATE posts SET
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            excerpt = COALESCE($3, excerpt),
            status = COALESCE($4, status),
            updated_at = NOW()
        WHERE id = $5
        RETURNING *
        "#,
        input.title,
        input.content,
        input.excerpt,
        input.status,
        id
    )
    .fetch_one(state.db())
    .await?;

    Ok(Json(post))
}
```

### Delete Handler
```rust
pub async fn delete_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    // Check ownership or admin
    let existing = sqlx::query!("SELECT author_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db())
        .await?
        .ok_or(ApiError::NotFound)?;

    if existing.author_id != user.id && !user.is_admin() {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!("DELETE FROM posts WHERE id = $1", id)
        .execute(state.db())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
```

---

## EXTRACTORS

### Built-in Axum Extractors
```rust
use axum::extract::{State, Path, Query, Json, Request, ConnectInfo};
use std::net::SocketAddr;

// Application state
State(state): State<AppState>

// Single path parameter
Path(id): Path<i64>

// Multiple path parameters
Path((org, repo)): Path<(String, String)>

// Named path parameters (struct)
#[derive(Deserialize)]
struct PathParams {
    org: String,
    repo: String,
}
Path(params): Path<PathParams>

// Query string parameters
Query(params): Query<ListParams>

// JSON request body
Json(body): Json<CreateInput>

// Full request access
req: Request

// Client IP address
ConnectInfo(addr): ConnectInfo<SocketAddr>

// Request headers
headers: HeaderMap
```

### Custom Extractor: AuthUser
```rust
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        // Verify JWT and get user
        let claims = verify_jwt(token).map_err(|_| ApiError::Unauthorized)?;
        let user = get_user_by_id(claims.sub).await?;

        Ok(AuthUser(user))
    }
}
```

### Custom Extractor: ValidatedJson
```rust
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Parse JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ApiError::BadRequest(format!("Invalid JSON: {}", e)))?;

        // Validate
        value.validate().map_err(ApiError::Validation)?;

        Ok(ValidatedJson(value))
    }
}
```

### Custom Extractor: Pagination
```rust
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            offset: 0,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");
        let params: HashMap<String, String> =
            serde_urlencoded::from_str(query).unwrap_or_default();

        let page = params
            .get("page")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
            .max(1);

        let per_page = params
            .get("per_page")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20)
            .min(100);

        Ok(Pagination {
            page,
            per_page,
            offset: (page - 1) * per_page,
        })
    }
}
```

### Custom Extractor: OptionalAuthUser
```rust
pub struct OptionalAuthUser(pub Option<User>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .and_then(|token| verify_jwt(token).ok())
            .and_then(|claims| get_user_by_id_sync(claims.sub).ok());

        Ok(OptionalAuthUser(user))
    }
}
```

---

## MIDDLEWARE

### Middleware Stack (Recommended Order)
```rust
use axum::middleware;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
};

pub fn apply_middleware(router: Router<AppState>) -> Router<AppState> {
    router.layer(
        ServiceBuilder::new()
            // 1. Response compression (outermost - runs last on response)
            .layer(CompressionLayer::new())
            // 2. Request tracing/logging
            .layer(TraceLayer::new_for_http())
            // 3. Add request ID
            .layer(middleware::from_fn(request_id))
            // 4. Security headers
            .layer(middleware::from_fn(security_headers))
            // 5. Rate limiting
            .layer(middleware::from_fn(rate_limit))
            // 6. CORS handling
            .layer(CorsLayer::permissive())
            // 7. Body size limit (10MB)
            .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
    )
}
```

### Request ID Middleware
```rust
use uuid::Uuid;

#[derive(Clone)]
pub struct RequestId(pub String);

pub async fn request_id(
    mut req: Request,
    next: Next,
) -> Response {
    let id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(RequestId(id.clone()));

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "X-Request-ID",
        id.parse().unwrap(),
    );

    response
}
```

### Security Headers Middleware
```rust
pub async fn security_headers(
    req: Request,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'".parse().unwrap(),
    );

    response
}
```

### Rate Limiting Middleware
```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }
}

pub async fn rate_limit(
    State(limiter): State<RateLimiter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let ip = addr.ip().to_string();
    let now = Instant::now();

    let mut requests = limiter.requests.write().await;

    let (count, started) = requests
        .entry(ip.clone())
        .or_insert((0, now));

    if now.duration_since(*started) > limiter.window {
        *count = 1;
        *started = now;
    } else {
        *count += 1;
    }

    if *count > limiter.max_requests {
        return Err(ApiError::TooManyRequests);
    }

    drop(requests);
    Ok(next.run(req).await)
}
```

### Authentication Middleware
```rust
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;

    let claims = verify_jwt(token).map_err(|_| ApiError::Unauthorized)?;
    let user = get_user_by_id(&state, claims.sub)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub async fn require_admin(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let user = req
        .extensions()
        .get::<User>()
        .ok_or(ApiError::Unauthorized)?;

    if !user.is_admin() {
        return Err(ApiError::Forbidden);
    }

    Ok(next.run(req).await)
}

// Apply to routes
fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/profile", get(get_profile).put(update_profile))
        .route("/settings", get(get_settings).put(update_settings))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
}
```

---

## ERROR HANDLING

### Error Type Definition
```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde_json::json;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Unauthorized,
    Forbidden,
    BadRequest(String),
    Validation(ValidationErrors),
    Conflict(String),
    TooManyRequests,
    Internal(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Resource not found".to_string(),
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required".to_string(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Access denied".to_string(),
            ),
            Self::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg,
            ),
            Self::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                format_validation_errors(&errors),
            ),
            Self::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                msg,
            ),
            Self::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many requests".to_string(),
            ),
            Self::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                )
            }
        };

        (
            status,
            Json(json!({
                "error": {
                    "code": code,
                    "message": message,
                }
            })),
        )
            .into_response()
    }
}

// Automatic conversion from common error types
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(e.into()),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(e: ValidationErrors) -> Self {
        Self::Validation(e)
    }
}
```

---

## RESPONSE TYPES

### JSON Response
```rust
pub async fn handler() -> Json<Data> {
    Json(Data { field: "value".to_string() })
}
```

### JSON with Status Code
```rust
pub async fn create() -> (StatusCode, Json<Data>) {
    (StatusCode::CREATED, Json(data))
}
```

### No Content (204)
```rust
pub async fn delete() -> StatusCode {
    StatusCode::NO_CONTENT
}
```

### HTML Response
```rust
use axum::response::Html;

pub async fn page() -> Html<String> {
    Html("<html><body>Hello</body></html>".to_string())
}
```

### Redirect
```rust
use axum::response::Redirect;

pub async fn redirect_handler() -> Redirect {
    Redirect::to("/new-location")
}

pub async fn redirect_permanent() -> Redirect {
    Redirect::permanent("/new-location")
}
```

### Streaming Response (SSE)
```rust
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;

pub async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        for i in 0..10 {
            yield Ok(Event::default().data(format!("Message {}", i)));
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )
}
```

### File Download
```rust
use axum::body::Body;
use tokio_util::io::ReaderStream;

pub async fn download(Path(filename): Path<String>) -> Result<Response, ApiError> {
    let file = tokio::fs::File::open(&filename).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .header("Content-Type", "application/octet-stream")
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .body(body)
        .unwrap())
}
```

---

## DATABASE OPERATIONS (SQLx)

### Single Row Query
```rust
let post = sqlx::query_as!(Post, "SELECT * FROM posts WHERE id = $1", id)
    .fetch_one(state.db())
    .await?;
```

### Optional Row Query
```rust
let post = sqlx::query_as!(Post, "SELECT * FROM posts WHERE id = $1", id)
    .fetch_optional(state.db())
    .await?
    .ok_or(ApiError::NotFound)?;
```

### Multiple Rows Query
```rust
let posts = sqlx::query_as!(Post, "SELECT * FROM posts ORDER BY created_at DESC LIMIT $1", limit)
    .fetch_all(state.db())
    .await?;
```

### Insert with Returning
```rust
let post = sqlx::query_as!(
    Post,
    r#"
    INSERT INTO posts (title, content, author_id)
    VALUES ($1, $2, $3)
    RETURNING *
    "#,
    title,
    content,
    author_id
)
.fetch_one(state.db())
.await?;
```

### Update
```rust
let rows_affected = sqlx::query!(
    "UPDATE posts SET title = $1, updated_at = NOW() WHERE id = $2",
    new_title,
    id
)
.execute(state.db())
.await?
.rows_affected();
```

### Delete
```rust
sqlx::query!("DELETE FROM posts WHERE id = $1", id)
    .execute(state.db())
    .await?;
```

### Transactions
```rust
let mut tx = state.db().begin().await?;

// Multiple operations in transaction
let post = sqlx::query_as!(Post,
    "INSERT INTO posts (title, content) VALUES ($1, $2) RETURNING *",
    title, content
)
.fetch_one(&mut *tx)
.await?;

sqlx::query!(
    "INSERT INTO audit_log (action, entity_id) VALUES ($1, $2)",
    "post_created",
    post.id
)
.execute(&mut *tx)
.await?;

// Commit transaction
tx.commit().await?;

// If any error occurs, transaction is automatically rolled back
```

### Dynamic Queries
```rust
use sqlx::QueryBuilder;

let mut builder = QueryBuilder::new("SELECT * FROM posts WHERE 1=1");

if let Some(status) = &params.status {
    builder.push(" AND status = ").push_bind(status);
}

if let Some(search) = &params.search {
    builder.push(" AND title ILIKE '%' || ").push_bind(search).push(" || '%'");
}

builder.push(" ORDER BY created_at DESC LIMIT ").push_bind(limit);

let posts = builder
    .build_query_as::<Post>()
    .fetch_all(state.db())
    .await?;
```

---

## CACHING (Redis)

### Get Cached Value
```rust
pub async fn get_cached<T: DeserializeOwned>(
    redis: &RedisPool,
    key: &str,
) -> Option<T> {
    let mut conn = redis.get().await.ok()?;
    let data: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(&mut conn)
        .await
        .ok()?;

    data.and_then(|s| serde_json::from_str(&s).ok())
}
```

### Set Cached Value
```rust
pub async fn set_cached<T: Serialize>(
    redis: &RedisPool,
    key: &str,
    value: &T,
    ttl_secs: u64,
) -> Result<(), RedisError> {
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
```

### Delete Cached Value
```rust
pub async fn delete_cached(redis: &RedisPool, key: &str) -> Result<(), RedisError> {
    let mut conn = redis.get().await?;
    redis::cmd("DEL").arg(key).query_async(&mut conn).await?;
    Ok(())
}
```

### Cache Pattern in Handler
```rust
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Post>, ApiError> {
    let cache_key = format!("post:{}", id);

    // Try cache first
    if let Some(post) = get_cached::<Post>(state.redis(), &cache_key).await {
        return Ok(Json(post));
    }

    // Query database
    let post = sqlx::query_as!(Post, "SELECT * FROM posts WHERE id = $1", id)
        .fetch_optional(state.db())
        .await?
        .ok_or(ApiError::NotFound)?;

    // Cache for 5 minutes
    let _ = set_cached(state.redis(), &cache_key, &post, 300).await;

    Ok(Json(post))
}

// Invalidate cache on update
pub async fn update_post(...) -> Result<Json<Post>, ApiError> {
    // ... update logic ...

    // Invalidate cache
    let cache_key = format!("post:{}", id);
    let _ = delete_cached(state.redis(), &cache_key).await;

    Ok(Json(post))
}
```

---

## COMMON ROUTE PATTERNS

### CRUD Resource
```rust
fn resource_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(show).put(update).delete(destroy))
}
```

### Nested Resources
```rust
fn nested_routes() -> Router<AppState> {
    Router::new()
        .route("/posts/:post_id/comments", get(list_comments).post(create_comment))
        .route("/posts/:post_id/comments/:id", get(get_comment).put(update_comment).delete(delete_comment))
}
```

### Versioned API
```rust
Router::new()
    .nest("/api/v1", api_v1_routes())
    .nest("/api/v2", api_v2_routes())
```

### Public + Protected Routes
```rust
fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Public routes
        .route("/posts", get(list_posts))
        .route("/posts/:id", get(get_post))
        // Protected routes
        .nest("/admin", admin_routes().layer(
            middleware::from_fn_with_state(state.clone(), require_auth)
        ))
}
```

---

## BEST PRACTICES

1. **Use extractors for common patterns** - Don't parse manually
2. **Validate input with `validator` crate** - Use `ValidatedJson<T>`
3. **Return proper HTTP status codes** - 201 for create, 204 for delete
4. **Handle errors consistently** - Use a single `ApiError` enum
5. **Use transactions for multi-step operations** - Maintain data integrity
6. **Cache expensive queries** - Use Redis with appropriate TTLs
7. **Add request tracing/logging** - Use `TraceLayer`
8. **Apply rate limiting** - Protect against abuse
9. **Use prepared statements (SQLx)** - Prevents SQL injection
10. **Keep handlers thin** - Move business logic to services
