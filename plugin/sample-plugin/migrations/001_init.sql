-- Hello World Plugin - Initial Migration
-- This is optional - only needed if your plugin stores data

-- Example: Create a greetings log table
CREATE TABLE IF NOT EXISTS hello_world_log (
    id BIGSERIAL PRIMARY KEY,
    greeting VARCHAR(255) NOT NULL,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for querying by date
CREATE INDEX idx_hello_world_log_created_at ON hello_world_log(created_at DESC);
