# RustPress Analytics Plugin

A comprehensive analytics plugin demonstrating advanced RustPress plugin development patterns.

## Features

- **Page View Tracking**: Automatic tracking of all page views with visitor/session management
- **Event Tracking**: Custom events for downloads, outbound links, and user actions
- **Real-time Analytics**: Live visitor monitoring with WebSocket updates
- **Reports**: Overview, pages, referrers, devices, and geography reports
- **Data Export**: CSV, JSON, and PDF export capabilities
- **Privacy Compliant**: Configurable data retention and anonymization options

## Architecture

```
advanced-plugin/
├── plugin.toml          # Plugin manifest with settings, API, cron, CLI
├── Cargo.toml           # Rust dependencies
├── migrations/          # Database migrations
│   └── 001_init.sql     # Initial schema
└── src/
    ├── lib.rs           # Main plugin entry point
    ├── models/          # Data models and DTOs
    │   └── mod.rs
    ├── services/        # Business logic
    │   └── mod.rs       # Tracking, Analytics, Report services
    ├── api/             # REST API handlers
    │   └── mod.rs
    └── hooks/           # Action and filter handlers
        └── mod.rs
```

## Key Patterns Demonstrated

### 1. Plugin Configuration
- Comprehensive settings schema with validation
- Multiple setting types (boolean, integer, string, array)
- Default values and constraints

### 2. Service Architecture
- Separation of concerns (Tracking, Analytics, Reports)
- Async service initialization
- Resource cleanup on deactivation

### 3. API Design
- RESTful endpoints with proper status codes
- Query parameter handling for reports
- Error responses with JSON

### 4. Hook System
- Action hooks for page views, logins, content publish
- Filter hooks for script injection
- Cron jobs for aggregation and cleanup

### 5. Database Operations
- SQLx for type-safe queries
- Proper indexing strategy
- Data aggregation patterns

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/analytics/track` | Track pageview or event |
| GET | `/api/v1/analytics/pageviews` | Get pageview data |
| GET | `/api/v1/analytics/visitors` | Get visitor statistics |
| GET | `/api/v1/analytics/realtime` | Get real-time visitors |
| GET | `/api/v1/analytics/reports/overview` | Overview report |
| GET | `/api/v1/analytics/reports/pages` | Top pages report |
| GET | `/api/v1/analytics/reports/referrers` | Referrer sources |
| GET | `/api/v1/analytics/reports/devices` | Device breakdown |
| GET | `/api/v1/analytics/reports/geography` | Geographic data |
| POST | `/api/v1/analytics/reports/export` | Export report data |

## Configuration Options

Key settings in the admin panel:

- **tracking_enabled**: Enable/disable all tracking
- **track_admins**: Include admin users in tracking
- **realtime_enabled**: Enable real-time visitor tracking
- **session_timeout**: Session expiration in minutes
- **data_retention_days**: How long to keep raw data
- **excluded_paths**: Paths to ignore (e.g., /admin/*)
- **excluded_ips**: IP addresses to exclude
- **track_outbound_links**: Track external link clicks
- **track_downloads**: Track file downloads
- **anonymize_ip**: Remove last octet for privacy

## Usage

### Installation

```bash
rustpress plugin install rustpress-analytics
```

### CLI Commands

```bash
# View real-time stats
rustpress analytics:stats

# Generate report
rustpress analytics:report --period=30d --format=csv

# Cleanup old data
rustpress analytics:cleanup --days=90
```

### JavaScript API

The plugin injects a tracking script that exposes `window.rpAnalytics`:

```javascript
// Track custom event
rpAnalytics.trackEvent('videos', 'play', 'intro-video', 1);

// Track page view manually (for SPAs)
rpAnalytics.trackPageView();
```

## License

MIT
