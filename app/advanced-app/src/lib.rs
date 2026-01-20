//! RustPress Blog API
//!
//! A comprehensive blog API demonstrating advanced app development patterns.

pub mod auth;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use rustpress_apps::prelude::*;
use std::sync::Arc;

/// Blog API Application
pub struct BlogApp {
    config: AppConfig,
    services: Option<Arc<BlogServices>>,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub posts_per_page: i64,
    pub comments_require_moderation: bool,
    pub allow_guest_comments: bool,
    pub max_comment_depth: i32,
    pub excerpt_length: usize,
    pub feed_items: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            posts_per_page: 10,
            comments_require_moderation: true,
            allow_guest_comments: true,
            max_comment_depth: 3,
            excerpt_length: 200,
            feed_items: 20,
        }
    }
}

/// Aggregated services container
pub struct BlogServices {
    pub posts: services::PostService,
    pub comments: services::CommentService,
    pub categories: services::CategoryService,
    pub tags: services::TagService,
    pub media: services::MediaService,
    pub search: services::SearchService,
    pub auth: Arc<auth::AuthService>,
}

#[rustpress_apps::app]
impl App for BlogApp {
    fn new() -> Self {
        Self {
            config: AppConfig::default(),
            services: None,
        }
    }

    async fn activate(&mut self, ctx: &AppContext) -> Result<(), AppError> {
        tracing::info!("Activating Blog API");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&ctx.db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Initialize auth configuration from environment
        let auth_config = auth::AuthConfig::from_env();
        auth_config
            .validate()
            .map_err(|e| AppError::Config(e))?;

        // Initialize auth service
        let auth_service = Arc::new(auth::AuthService::new(ctx.db.clone(), auth_config));

        // Initialize services
        let services = Arc::new(BlogServices {
            posts: services::PostService::new(ctx.db.clone(), ctx.cache.clone()),
            comments: services::CommentService::new(ctx.db.clone()),
            categories: services::CategoryService::new(ctx.db.clone(), ctx.cache.clone()),
            tags: services::TagService::new(ctx.db.clone(), ctx.cache.clone()),
            media: services::MediaService::new(ctx.db.clone(), ctx.storage.clone()),
            search: services::SearchService::new(ctx.db.clone()),
            auth: auth_service,
        });

        self.services = Some(services);

        tracing::info!("Blog API activated successfully");
        Ok(())
    }

    async fn deactivate(&mut self, _ctx: &AppContext) -> Result<(), AppError> {
        tracing::info!("Deactivating Blog API");
        self.services = None;
        Ok(())
    }

    fn routes(&self) -> Router {
        let services = self.services.clone().expect("Services not initialized");
        let auth_service = services.auth.clone();

        // Auth routes (public - no authentication required)
        let auth_routes = Router::new()
            .route("/auth/register", post(auth::handlers::register))
            .route("/auth/login", post(auth::handlers::login))
            .route("/auth/logout", post(auth::handlers::logout))
            .route("/auth/refresh", post(auth::handlers::refresh_token))
            .route("/auth/forgot-password", post(auth::handlers::forgot_password))
            .route("/auth/reset-password", post(auth::handlers::reset_password))
            .route("/auth/verify-email", post(auth::handlers::verify_email))
            .with_state(auth_service.clone());

        // Auth routes requiring authentication
        let auth_protected = Router::new()
            .route("/auth/me", get(auth::handlers::get_current_user))
            .route("/auth/change-password", post(auth::handlers::change_password))
            .route("/auth/resend-verification", post(auth::handlers::resend_verification))
            .layer(axum_middleware::from_fn(middleware::auth::require_auth))
            .with_state(auth_service);

        // Public routes
        let public = Router::new()
            .route("/posts", get(handlers::posts::list_posts))
            .route("/posts/:slug", get(handlers::posts::get_post_by_slug))
            .route("/posts/:id/comments", get(handlers::comments::list_comments))
            .route("/posts/:id/comments", post(handlers::comments::create_comment))
            .route("/categories", get(handlers::categories::list_categories))
            .route("/tags", get(handlers::tags::list_tags))
            .route("/feed", get(handlers::feed::rss_feed))
            .route("/search", get(handlers::search::search_posts))
            .layer(axum_middleware::from_fn(middleware::view_counter::increment_views));

        // Protected routes (require authentication)
        let protected = Router::new()
            .route("/posts", post(handlers::posts::create_post))
            .route("/posts/:id", put(handlers::posts::update_post))
            .route("/posts/:id", delete(handlers::posts::delete_post))
            .route("/posts/:id/publish", post(handlers::posts::publish_post))
            .route("/posts/:id/unpublish", post(handlers::posts::unpublish_post))
            .route("/drafts", get(handlers::posts::list_drafts))
            .route("/media", get(handlers::media::list_media))
            .route("/media", post(handlers::media::upload_media))
            .route("/media/:id", delete(handlers::media::delete_media))
            .route("/comments/:id/approve", post(handlers::comments::approve_comment))
            .route("/comments/:id/reject", post(handlers::comments::reject_comment))
            .route("/categories", post(handlers::categories::create_category))
            .route("/categories/:id", put(handlers::categories::update_category))
            .route("/categories/:id", delete(handlers::categories::delete_category))
            .route("/tags", post(handlers::tags::create_tag))
            .route("/tags/:id", put(handlers::tags::update_tag))
            .route("/tags/:id", delete(handlers::tags::delete_tag))
            .layer(axum_middleware::from_fn(middleware::auth::require_auth));

        // Admin routes
        let admin = Router::new()
            .route("/admin/posts", get(handlers::admin::list_all_posts))
            .route("/admin/comments/pending", get(handlers::admin::pending_comments))
            .route("/admin/stats", get(handlers::admin::blog_stats))
            .layer(axum_middleware::from_fn(middleware::auth::require_admin));

        // Merge all routes
        Router::new()
            .merge(auth_routes)
            .merge(auth_protected)
            .merge(public)
            .merge(protected)
            .merge(admin)
            .layer(axum_middleware::from_fn(middleware::cache::cache_response))
            .layer(axum_middleware::from_fn(middleware::rate_limit::rate_limiter))
            .with_state(services)
    }
}
