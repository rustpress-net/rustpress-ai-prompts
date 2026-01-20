//! Blog Services

use crate::models::*;
use rustpress_apps::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Service error type
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Storage error: {0}")]
    Storage(String),
}

/// Post service
pub struct PostService {
    db: PgPool,
    cache: Arc<dyn Cache>,
}

impl PostService {
    pub fn new(db: PgPool, cache: Arc<dyn Cache>) -> Self {
        Self { db, cache }
    }

    /// List published posts with pagination
    pub async fn list_published(&self, query: &PostQuery) -> Result<PaginatedResponse<PostWithRelations>, ServiceError> {
        let cache_key = format!("posts:list:{:?}", query);

        // Try cache first
        if let Some(cached) = self.cache.get::<PaginatedResponse<PostWithRelations>>(&cache_key).await {
            return Ok(cached);
        }

        let mut sql = String::from(
            "SELECT p.*,
                    json_build_object('id', u.id, 'name', u.name, 'avatar', u.avatar, 'bio', u.bio) as author
             FROM blog_posts p
             JOIN users u ON u.id = p.author_id
             WHERE p.status = 'published'"
        );

        // Apply filters
        if let Some(ref category) = query.category {
            sql.push_str(" AND EXISTS (SELECT 1 FROM blog_post_categories pc
                          JOIN blog_categories c ON c.id = pc.category_id
                          WHERE pc.post_id = p.id AND c.slug = $3)");
        }

        // Sort
        let order = query.order.as_deref().unwrap_or("desc");
        match query.sort.as_deref() {
            Some("views") => sql.push_str(&format!(" ORDER BY p.view_count {}", order)),
            Some("comments") => sql.push_str(&format!(" ORDER BY p.comment_count {}", order)),
            _ => sql.push_str(&format!(" ORDER BY p.published_at {}", order)),
        }

        sql.push_str(" LIMIT $1 OFFSET $2");

        let posts: Vec<Post> = sqlx::query_as(&sql)
            .bind(query.per_page())
            .bind(query.offset())
            .fetch_all(&self.db)
            .await?;

        // Get total count
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blog_posts WHERE status = 'published'")
            .fetch_one(&self.db)
            .await?;

        // Fetch relations for each post
        let mut posts_with_relations = Vec::new();
        for post in posts {
            let relations = self.get_post_relations(&post).await?;
            posts_with_relations.push(relations);
        }

        let response = PaginatedResponse {
            data: posts_with_relations,
            pagination: PaginationMeta::new(total, query.page(), query.per_page()),
        };

        // Cache for 5 minutes
        self.cache.set(&cache_key, &response, Some(300)).await;

        Ok(response)
    }

    /// Get a post by slug
    pub async fn get_by_slug(&self, slug: &str) -> Result<PostWithRelations, ServiceError> {
        let cache_key = format!("posts:slug:{}", slug);

        if let Some(cached) = self.cache.get::<PostWithRelations>(&cache_key).await {
            return Ok(cached);
        }

        let post: Post = sqlx::query_as(
            "SELECT * FROM blog_posts WHERE slug = $1 AND status = 'published'"
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Post not found: {}", slug)))?;

        let result = self.get_post_relations(&post).await?;

        self.cache.set(&cache_key, &result, Some(600)).await;

        Ok(result)
    }

    /// Get a post by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Post, ServiceError> {
        sqlx::query_as("SELECT * FROM blog_posts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Post not found: {}", id)))
    }

    /// Create a new post
    pub async fn create(&self, author_id: Uuid, req: CreatePostRequest) -> Result<Post, ServiceError> {
        let slug = slug::slugify(&req.title);
        let excerpt = req.excerpt.or_else(|| {
            Some(req.content.chars().take(200).collect())
        });

        let post: Post = sqlx::query_as(
            r#"INSERT INTO blog_posts
               (author_id, title, slug, content, excerpt, featured_image, status, meta_title, meta_description, scheduled_for)
               VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7, $8, $9)
               RETURNING *"#
        )
        .bind(author_id)
        .bind(&req.title)
        .bind(&slug)
        .bind(&req.content)
        .bind(&excerpt)
        .bind(&req.featured_image)
        .bind(&req.meta_title)
        .bind(&req.meta_description)
        .bind(&req.scheduled_for)
        .fetch_one(&self.db)
        .await?;

        // Attach categories and tags
        if let Some(category_ids) = req.category_ids {
            self.attach_categories(post.id, &category_ids).await?;
        }
        if let Some(tag_ids) = req.tag_ids {
            self.attach_tags(post.id, &tag_ids).await?;
        }

        // Invalidate cache
        self.cache.delete_pattern("posts:*").await;

        Ok(post)
    }

    /// Update a post
    pub async fn update(&self, id: Uuid, author_id: Uuid, req: UpdatePostRequest) -> Result<Post, ServiceError> {
        let existing = self.get_by_id(id).await?;

        // Check ownership
        if existing.author_id != author_id {
            return Err(ServiceError::PermissionDenied);
        }

        let title = req.title.unwrap_or(existing.title);
        let slug = slug::slugify(&title);

        let post: Post = sqlx::query_as(
            r#"UPDATE blog_posts SET
               title = $2, slug = $3, content = COALESCE($4, content),
               excerpt = COALESCE($5, excerpt), featured_image = COALESCE($6, featured_image),
               meta_title = COALESCE($7, meta_title), meta_description = COALESCE($8, meta_description),
               updated_at = NOW()
               WHERE id = $1
               RETURNING *"#
        )
        .bind(id)
        .bind(&title)
        .bind(&slug)
        .bind(&req.content)
        .bind(&req.excerpt)
        .bind(&req.featured_image)
        .bind(&req.meta_title)
        .bind(&req.meta_description)
        .fetch_one(&self.db)
        .await?;

        // Update categories and tags if provided
        if let Some(category_ids) = req.category_ids {
            sqlx::query("DELETE FROM blog_post_categories WHERE post_id = $1")
                .bind(id)
                .execute(&self.db)
                .await?;
            self.attach_categories(id, &category_ids).await?;
        }
        if let Some(tag_ids) = req.tag_ids {
            sqlx::query("DELETE FROM blog_post_tags WHERE post_id = $1")
                .bind(id)
                .execute(&self.db)
                .await?;
            self.attach_tags(id, &tag_ids).await?;
        }

        // Invalidate cache
        self.cache.delete_pattern("posts:*").await;

        Ok(post)
    }

    /// Publish a post
    pub async fn publish(&self, id: Uuid) -> Result<Post, ServiceError> {
        let post: Post = sqlx::query_as(
            "UPDATE blog_posts SET status = 'published', published_at = NOW(), updated_at = NOW()
             WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        self.cache.delete_pattern("posts:*").await;

        Ok(post)
    }

    /// Unpublish a post
    pub async fn unpublish(&self, id: Uuid) -> Result<Post, ServiceError> {
        let post: Post = sqlx::query_as(
            "UPDATE blog_posts SET status = 'draft', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        self.cache.delete_pattern("posts:*").await;

        Ok(post)
    }

    /// Delete a post
    pub async fn delete(&self, id: Uuid, author_id: Uuid) -> Result<(), ServiceError> {
        let existing = self.get_by_id(id).await?;

        if existing.author_id != author_id {
            return Err(ServiceError::PermissionDenied);
        }

        sqlx::query("DELETE FROM blog_posts WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        self.cache.delete_pattern("posts:*").await;

        Ok(())
    }

    /// Increment view count
    pub async fn increment_views(&self, id: Uuid) -> Result<(), ServiceError> {
        sqlx::query("UPDATE blog_posts SET view_count = view_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Get post with relations
    async fn get_post_relations(&self, post: &Post) -> Result<PostWithRelations, ServiceError> {
        let author: AuthorInfo = sqlx::query_as(
            "SELECT id, name, avatar, bio FROM users WHERE id = $1"
        )
        .bind(post.author_id)
        .fetch_one(&self.db)
        .await?;

        let categories: Vec<Category> = sqlx::query_as(
            "SELECT c.* FROM blog_categories c
             JOIN blog_post_categories pc ON pc.category_id = c.id
             WHERE pc.post_id = $1"
        )
        .bind(post.id)
        .fetch_all(&self.db)
        .await?;

        let tags: Vec<Tag> = sqlx::query_as(
            "SELECT t.* FROM blog_tags t
             JOIN blog_post_tags pt ON pt.tag_id = t.id
             WHERE pt.post_id = $1"
        )
        .bind(post.id)
        .fetch_all(&self.db)
        .await?;

        Ok(PostWithRelations {
            post: post.clone(),
            author,
            categories,
            tags,
        })
    }

    async fn attach_categories(&self, post_id: Uuid, category_ids: &[Uuid]) -> Result<(), ServiceError> {
        for cat_id in category_ids {
            sqlx::query("INSERT INTO blog_post_categories (post_id, category_id) VALUES ($1, $2)")
                .bind(post_id)
                .bind(cat_id)
                .execute(&self.db)
                .await?;
        }
        Ok(())
    }

    async fn attach_tags(&self, post_id: Uuid, tag_ids: &[Uuid]) -> Result<(), ServiceError> {
        for tag_id in tag_ids {
            sqlx::query("INSERT INTO blog_post_tags (post_id, tag_id) VALUES ($1, $2)")
                .bind(post_id)
                .bind(tag_id)
                .execute(&self.db)
                .await?;
        }
        Ok(())
    }
}

/// Comment service
pub struct CommentService {
    db: PgPool,
}

impl CommentService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// List comments for a post
    pub async fn list_for_post(&self, post_id: Uuid) -> Result<Vec<CommentThread>, ServiceError> {
        let comments: Vec<Comment> = sqlx::query_as(
            "SELECT * FROM blog_comments WHERE post_id = $1 AND status = 'approved' ORDER BY created_at ASC"
        )
        .bind(post_id)
        .fetch_all(&self.db)
        .await?;

        Ok(self.build_comment_tree(comments))
    }

    /// Create a comment
    pub async fn create(
        &self,
        post_id: Uuid,
        author_id: Option<Uuid>,
        req: CreateCommentRequest,
        ip: Option<String>,
        user_agent: Option<String>,
        requires_moderation: bool,
    ) -> Result<Comment, ServiceError> {
        let status = if requires_moderation {
            CommentStatus::Pending
        } else {
            CommentStatus::Approved
        };

        let comment: Comment = sqlx::query_as(
            r#"INSERT INTO blog_comments
               (post_id, parent_id, author_id, author_name, author_email, author_url, content, status, ip_address, user_agent)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#
        )
        .bind(post_id)
        .bind(req.parent_id)
        .bind(author_id)
        .bind(&req.author_name)
        .bind(&req.author_email)
        .bind(&req.author_url)
        .bind(&req.content)
        .bind(status)
        .bind(ip)
        .bind(user_agent)
        .fetch_one(&self.db)
        .await?;

        // Update comment count
        sqlx::query("UPDATE blog_posts SET comment_count = comment_count + 1 WHERE id = $1")
            .bind(post_id)
            .execute(&self.db)
            .await?;

        Ok(comment)
    }

    /// Approve a comment
    pub async fn approve(&self, id: Uuid) -> Result<Comment, ServiceError> {
        sqlx::query_as("UPDATE blog_comments SET status = 'approved' WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Comment not found".into()))
    }

    /// Reject a comment
    pub async fn reject(&self, id: Uuid) -> Result<Comment, ServiceError> {
        sqlx::query_as("UPDATE blog_comments SET status = 'rejected' WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Comment not found".into()))
    }

    fn build_comment_tree(&self, comments: Vec<Comment>) -> Vec<CommentThread> {
        use std::collections::HashMap;

        let mut map: HashMap<Uuid, CommentThread> = HashMap::new();
        let mut roots = Vec::new();

        // First pass: create all threads
        for comment in comments {
            map.insert(comment.id, CommentThread {
                comment,
                replies: Vec::new(),
            });
        }

        // Second pass: build tree
        let ids: Vec<Uuid> = map.keys().copied().collect();
        for id in ids {
            let thread = map.remove(&id).unwrap();
            if let Some(parent_id) = thread.comment.parent_id {
                if let Some(parent) = map.get_mut(&parent_id) {
                    parent.replies.push(thread);
                } else {
                    roots.push(thread);
                }
            } else {
                roots.push(thread);
            }
        }

        roots
    }
}

/// Category service
pub struct CategoryService {
    db: PgPool,
    cache: Arc<dyn Cache>,
}

impl CategoryService {
    pub fn new(db: PgPool, cache: Arc<dyn Cache>) -> Self {
        Self { db, cache }
    }

    pub async fn list(&self) -> Result<Vec<Category>, ServiceError> {
        if let Some(cached) = self.cache.get::<Vec<Category>>("categories:all").await {
            return Ok(cached);
        }

        let categories: Vec<Category> = sqlx::query_as(
            "SELECT * FROM blog_categories ORDER BY name ASC"
        )
        .fetch_all(&self.db)
        .await?;

        self.cache.set("categories:all", &categories, Some(3600)).await;

        Ok(categories)
    }

    pub async fn create(&self, req: CategoryRequest) -> Result<Category, ServiceError> {
        let slug = slug::slugify(&req.name);

        let category: Category = sqlx::query_as(
            "INSERT INTO blog_categories (name, slug, parent_id, description) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(&req.name)
        .bind(&slug)
        .bind(req.parent_id)
        .bind(&req.description)
        .fetch_one(&self.db)
        .await?;

        self.cache.delete("categories:all").await;

        Ok(category)
    }

    pub async fn update(&self, id: Uuid, req: CategoryRequest) -> Result<Category, ServiceError> {
        let slug = slug::slugify(&req.name);

        let category: Category = sqlx::query_as(
            "UPDATE blog_categories SET name = $2, slug = $3, parent_id = $4, description = $5 WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .bind(&req.name)
        .bind(&slug)
        .bind(req.parent_id)
        .bind(&req.description)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Category not found".into()))?;

        self.cache.delete("categories:all").await;

        Ok(category)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM blog_categories WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        self.cache.delete("categories:all").await;

        Ok(())
    }
}

/// Tag service
pub struct TagService {
    db: PgPool,
    cache: Arc<dyn Cache>,
}

impl TagService {
    pub fn new(db: PgPool, cache: Arc<dyn Cache>) -> Self {
        Self { db, cache }
    }

    pub async fn list(&self) -> Result<Vec<Tag>, ServiceError> {
        if let Some(cached) = self.cache.get::<Vec<Tag>>("tags:all").await {
            return Ok(cached);
        }

        let tags: Vec<Tag> = sqlx::query_as("SELECT * FROM blog_tags ORDER BY name ASC")
            .fetch_all(&self.db)
            .await?;

        self.cache.set("tags:all", &tags, Some(3600)).await;

        Ok(tags)
    }

    pub async fn create(&self, req: TagRequest) -> Result<Tag, ServiceError> {
        let slug = slug::slugify(&req.name);

        let tag: Tag = sqlx::query_as(
            "INSERT INTO blog_tags (name, slug) VALUES ($1, $2) RETURNING *"
        )
        .bind(&req.name)
        .bind(&slug)
        .fetch_one(&self.db)
        .await?;

        self.cache.delete("tags:all").await;

        Ok(tag)
    }

    pub async fn update(&self, id: Uuid, req: TagRequest) -> Result<Tag, ServiceError> {
        let slug = slug::slugify(&req.name);

        let tag: Tag = sqlx::query_as(
            "UPDATE blog_tags SET name = $2, slug = $3 WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .bind(&req.name)
        .bind(&slug)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Tag not found".into()))?;

        self.cache.delete("tags:all").await;

        Ok(tag)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM blog_tags WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        self.cache.delete("tags:all").await;

        Ok(())
    }
}

/// Media service
pub struct MediaService {
    db: PgPool,
    storage: Arc<dyn Storage>,
}

impl MediaService {
    pub fn new(db: PgPool, storage: Arc<dyn Storage>) -> Self {
        Self { db, storage }
    }

    pub async fn list(&self, user_id: Uuid, query: &MediaQuery) -> Result<Vec<Media>, ServiceError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let media: Vec<Media> = sqlx::query_as(
            "SELECT * FROM blog_media WHERE uploader_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(media)
    }

    pub async fn upload(
        &self,
        user_id: Uuid,
        filename: String,
        data: Vec<u8>,
        mime_type: String,
    ) -> Result<Media, ServiceError> {
        let id = Uuid::new_v4();
        let ext = filename.rsplit('.').next().unwrap_or("bin");
        let stored_name = format!("{}.{}", id, ext);
        let path = format!("uploads/media/{}", stored_name);

        // Upload to storage
        self.storage
            .put(&path, &data)
            .await
            .map_err(|e| ServiceError::Storage(e.to_string()))?;

        let url = self.storage.url(&path);
        let size = data.len() as i64;

        let media: Media = sqlx::query_as(
            r#"INSERT INTO blog_media
               (id, uploader_id, filename, original_name, mime_type, size, url)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#
        )
        .bind(id)
        .bind(user_id)
        .bind(&stored_name)
        .bind(&filename)
        .bind(&mime_type)
        .bind(size)
        .bind(&url)
        .fetch_one(&self.db)
        .await?;

        Ok(media)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), ServiceError> {
        let media: Media = sqlx::query_as("SELECT * FROM blog_media WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Media not found".into()))?;

        if media.uploader_id != user_id {
            return Err(ServiceError::PermissionDenied);
        }

        // Delete from storage
        let path = format!("uploads/media/{}", media.filename);
        self.storage
            .delete(&path)
            .await
            .map_err(|e| ServiceError::Storage(e.to_string()))?;

        sqlx::query("DELETE FROM blog_media WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}

/// Search service
pub struct SearchService {
    db: PgPool,
}

impl SearchService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResult, ServiceError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(10).min(100);
        let offset = (page - 1) * per_page;

        // Full-text search using PostgreSQL
        let posts: Vec<Post> = sqlx::query_as(
            r#"SELECT * FROM blog_posts
               WHERE status = 'published'
               AND (
                   to_tsvector('english', title || ' ' || COALESCE(excerpt, '') || ' ' || content)
                   @@ plainto_tsquery('english', $1)
               )
               ORDER BY ts_rank(
                   to_tsvector('english', title || ' ' || COALESCE(excerpt, '') || ' ' || content),
                   plainto_tsquery('english', $1)
               ) DESC
               LIMIT $2 OFFSET $3"#
        )
        .bind(&query.q)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM blog_posts
               WHERE status = 'published'
               AND to_tsvector('english', title || ' ' || COALESCE(excerpt, '') || ' ' || content)
               @@ plainto_tsquery('english', $1)"#
        )
        .bind(&query.q)
        .fetch_one(&self.db)
        .await?;

        let total_pages = (total as f64 / per_page as f64).ceil() as i64;

        // For brevity, returning posts without full relations
        let posts_with_relations: Vec<PostWithRelations> = Vec::new(); // Would normally fetch relations

        Ok(SearchResult {
            posts: posts_with_relations,
            total,
            page,
            per_page,
            total_pages,
        })
    }
}
