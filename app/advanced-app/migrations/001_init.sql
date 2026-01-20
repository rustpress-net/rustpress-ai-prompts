-- RustPress Blog API - Initial Schema
--
-- IMPORTANT: This migration requires the rustpress-auth plugin to be activated first.
-- The auth plugin creates the following tables that this schema references:
--   - users (with user_role and user_status enums)
--   - refresh_tokens
--   - password_reset_tokens
--   - email_verification_tokens
--
-- If running without the auth plugin, uncomment the user tables section below.

-- Post status enum
CREATE TYPE post_status AS ENUM ('draft', 'published', 'scheduled', 'archived');

-- Comment status enum
CREATE TYPE comment_status AS ENUM ('pending', 'approved', 'rejected', 'spam');

-- Blog posts table
CREATE TABLE IF NOT EXISTS blog_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    slug VARCHAR(250) NOT NULL UNIQUE,
    content TEXT NOT NULL,
    excerpt TEXT,
    featured_image VARCHAR(500),
    status post_status DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    scheduled_for TIMESTAMPTZ,
    view_count BIGINT DEFAULT 0,
    comment_count INTEGER DEFAULT 0,
    meta_title VARCHAR(70),
    meta_description VARCHAR(160),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Categories table
CREATE TABLE IF NOT EXISTS blog_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES blog_categories(id) ON DELETE SET NULL,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(120) NOT NULL UNIQUE,
    description TEXT,
    post_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tags table
CREATE TABLE IF NOT EXISTS blog_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    slug VARCHAR(60) NOT NULL UNIQUE,
    post_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Post-Category relationship
CREATE TABLE IF NOT EXISTS blog_post_categories (
    post_id UUID NOT NULL REFERENCES blog_posts(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES blog_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, category_id)
);

-- Post-Tag relationship
CREATE TABLE IF NOT EXISTS blog_post_tags (
    post_id UUID NOT NULL REFERENCES blog_posts(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES blog_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, tag_id)
);

-- Comments table
CREATE TABLE IF NOT EXISTS blog_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES blog_posts(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES blog_comments(id) ON DELETE CASCADE,
    author_id UUID REFERENCES users(id) ON DELETE SET NULL,
    author_name VARCHAR(100) NOT NULL,
    author_email VARCHAR(255) NOT NULL,
    author_url VARCHAR(500),
    content TEXT NOT NULL,
    status comment_status DEFAULT 'pending',
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Media files table
CREATE TABLE IF NOT EXISTS blog_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploader_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    width INTEGER,
    height INTEGER,
    alt_text VARCHAR(255),
    caption TEXT,
    url VARCHAR(500) NOT NULL,
    thumbnail_url VARCHAR(500),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for posts
CREATE INDEX idx_posts_author ON blog_posts(author_id);
CREATE INDEX idx_posts_status ON blog_posts(status);
CREATE INDEX idx_posts_published ON blog_posts(published_at DESC) WHERE status = 'published';
CREATE INDEX idx_posts_slug ON blog_posts(slug);
CREATE INDEX idx_posts_created ON blog_posts(created_at DESC);

-- Full-text search index
CREATE INDEX idx_posts_search ON blog_posts USING gin(
    to_tsvector('english', title || ' ' || COALESCE(excerpt, '') || ' ' || content)
);

-- Indexes for categories
CREATE INDEX idx_categories_parent ON blog_categories(parent_id);
CREATE INDEX idx_categories_slug ON blog_categories(slug);

-- Indexes for tags
CREATE INDEX idx_tags_slug ON blog_tags(slug);

-- Indexes for comments
CREATE INDEX idx_comments_post ON blog_comments(post_id);
CREATE INDEX idx_comments_parent ON blog_comments(parent_id);
CREATE INDEX idx_comments_status ON blog_comments(status);
CREATE INDEX idx_comments_created ON blog_comments(created_at DESC);

-- Indexes for media
CREATE INDEX idx_media_uploader ON blog_media(uploader_id);
CREATE INDEX idx_media_created ON blog_media(created_at DESC);
CREATE INDEX idx_media_type ON blog_media(mime_type);

-- Trigger to update post timestamps
CREATE OR REPLACE FUNCTION update_post_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER posts_updated_at
    BEFORE UPDATE ON blog_posts
    FOR EACH ROW
    EXECUTE FUNCTION update_post_timestamp();

-- Trigger to update category post count
CREATE OR REPLACE FUNCTION update_category_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE blog_categories SET post_count = post_count + 1 WHERE id = NEW.category_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE blog_categories SET post_count = post_count - 1 WHERE id = OLD.category_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER category_post_count
    AFTER INSERT OR DELETE ON blog_post_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_category_post_count();

-- Trigger to update tag post count
CREATE OR REPLACE FUNCTION update_tag_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE blog_tags SET post_count = post_count + 1 WHERE id = NEW.tag_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE blog_tags SET post_count = post_count - 1 WHERE id = OLD.tag_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tag_post_count
    AFTER INSERT OR DELETE ON blog_post_tags
    FOR EACH ROW
    EXECUTE FUNCTION update_tag_post_count();
