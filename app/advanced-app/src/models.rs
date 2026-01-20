//! Blog Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Post status enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Scheduled,
    Archived,
}

/// Comment status enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "comment_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Pending,
    Approved,
    Rejected,
    Spam,
}

/// Blog post
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: PostStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub view_count: i64,
    pub comment_count: i32,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Post with related data for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithRelations {
    #[serde(flatten)]
    pub post: Post,
    pub author: AuthorInfo,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

/// Minimal author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

/// Create post request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: String,

    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,

    #[validate(length(max = 500))]
    pub excerpt: Option<String>,

    pub featured_image: Option<String>,

    pub category_ids: Option<Vec<Uuid>>,

    pub tag_ids: Option<Vec<Uuid>>,

    #[validate(length(max = 70))]
    pub meta_title: Option<String>,

    #[validate(length(max = 160))]
    pub meta_description: Option<String>,

    pub scheduled_for: Option<DateTime<Utc>>,
}

/// Update post request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,

    pub content: Option<String>,

    #[validate(length(max = 500))]
    pub excerpt: Option<String>,

    pub featured_image: Option<String>,

    pub category_ids: Option<Vec<Uuid>>,

    pub tag_ids: Option<Vec<Uuid>>,

    #[validate(length(max = 70))]
    pub meta_title: Option<String>,

    #[validate(length(max = 160))]
    pub meta_description: Option<String>,
}

/// Post query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PostQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub author: Option<Uuid>,
    pub status: Option<PostStatus>,
    pub sort: Option<String>,  // "date", "views", "comments"
    pub order: Option<String>, // "asc", "desc"
}

impl PostQuery {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(10).min(100).max(1)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

/// Category
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Create/Update category request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    pub parent_id: Option<Uuid>,

    #[validate(length(max = 500))]
    pub description: Option<String>,
}

/// Tag
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Create/Update tag request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct TagRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

/// Comment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub author_email: String,
    pub author_url: Option<String>,
    pub content: String,
    pub status: CommentStatus,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Comment with nested replies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentThread {
    #[serde(flatten)]
    pub comment: Comment,
    pub replies: Vec<CommentThread>,
}

/// Create comment request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateCommentRequest {
    pub parent_id: Option<Uuid>,

    #[validate(length(min = 1, max = 100))]
    pub author_name: String,

    #[validate(email)]
    pub author_email: String,

    #[validate(url)]
    pub author_url: Option<String>,

    #[validate(length(min = 1, max = 10000))]
    pub content: String,
}

/// Media file
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Media query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct MediaQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub mime_type: Option<String>,
    pub search: Option<String>,
}

/// Search query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category: Option<String>,
    pub tag: Option<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub posts: Vec<PostWithRelations>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            total,
            page,
            per_page,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Blog statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogStats {
    pub total_posts: i64,
    pub published_posts: i64,
    pub draft_posts: i64,
    pub total_comments: i64,
    pub pending_comments: i64,
    pub total_categories: i64,
    pub total_tags: i64,
    pub total_media: i64,
    pub total_views: i64,
}

/// API error response
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
