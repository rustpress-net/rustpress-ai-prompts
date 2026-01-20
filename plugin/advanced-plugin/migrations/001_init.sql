-- RustPress Analytics - Initial Schema

-- Sessions table
CREATE TABLE IF NOT EXISTS analytics_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    visitor_id UUID NOT NULL,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ DEFAULT NOW(),
    page_views INTEGER DEFAULT 0,
    duration_seconds INTEGER,
    entry_page VARCHAR(500) NOT NULL,
    exit_page VARCHAR(500),
    device_type VARCHAR(50) NOT NULL,
    browser VARCHAR(100),
    os VARCHAR(100),
    country VARCHAR(2),
    city VARCHAR(100),
    is_bounce BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Page views table
CREATE TABLE IF NOT EXISTS analytics_pageviews (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES analytics_sessions(id) ON DELETE CASCADE,
    visitor_id UUID NOT NULL,
    path VARCHAR(500) NOT NULL,
    title VARCHAR(500),
    referrer VARCHAR(1000),
    utm_source VARCHAR(100),
    utm_medium VARCHAR(100),
    utm_campaign VARCHAR(100),
    ip_address VARCHAR(45),
    country VARCHAR(2),
    city VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Daily aggregated stats
CREATE TABLE IF NOT EXISTS analytics_daily_stats (
    date DATE PRIMARY KEY,
    page_views BIGINT DEFAULT 0,
    unique_visitors BIGINT DEFAULT 0,
    sessions BIGINT DEFAULT 0,
    bounce_rate DOUBLE PRECISION DEFAULT 0,
    avg_session_duration DOUBLE PRECISION DEFAULT 0,
    new_visitors BIGINT DEFAULT 0,
    returning_visitors BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sessions_visitor ON analytics_sessions(visitor_id);
CREATE INDEX idx_sessions_started ON analytics_sessions(started_at DESC);
CREATE INDEX idx_sessions_ended ON analytics_sessions(ended_at DESC);
CREATE INDEX idx_sessions_device ON analytics_sessions(device_type);
CREATE INDEX idx_sessions_country ON analytics_sessions(country);

CREATE INDEX idx_pageviews_session ON analytics_pageviews(session_id);
CREATE INDEX idx_pageviews_visitor ON analytics_pageviews(visitor_id);
CREATE INDEX idx_pageviews_path ON analytics_pageviews(path);
CREATE INDEX idx_pageviews_created ON analytics_pageviews(created_at DESC);
CREATE INDEX idx_pageviews_date ON analytics_pageviews((created_at::date));
