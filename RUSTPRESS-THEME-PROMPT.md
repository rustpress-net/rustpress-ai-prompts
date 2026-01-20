# RustPress Theme Development - AI Prompt Reference

You are building a theme for RustPress, a Rust-based CMS. Follow this specification exactly to create compatible themes.

---

## THEME MANIFEST (`theme.toml`)

Every theme MUST have a `theme.toml` manifest file at the root.

### Minimal Required Configuration
```toml
[theme]
id = "my-theme"               # Unique identifier (kebab-case)
name = "My Theme"             # Display name
version = "1.0.0"             # Semantic version
description = "Description"
author = "Author Name"
license = "GPL-2.0-or-later"
```

### Complete Configuration Schema
```toml
[theme]
id = "my-theme"
name = "My Theme"
version = "1.0.0"
description = "A beautiful, responsive theme"
author = "Author Name"
author_url = "https://example.com"
license = "GPL-2.0-or-later"
screenshot = "screenshot.png"         # Preview image (1200x900 recommended)
requires_rustpress = ">=1.0.0"        # Minimum RustPress version
tags = ["business", "modern", "responsive", "blog"]
theme_type = "hybrid"                 # classic|block|hybrid

# ============================================
# PARENT THEME (for child themes)
# ============================================
[parent]
id = "parent-theme-id"
min_version = "1.0.0"

# ============================================
# TEMPLATES CONFIGURATION
# ============================================
[templates]
directory = "templates"               # Templates folder
extension = "html"                    # Template file extension
parts_directory = "parts"             # Partial templates folder

# Template definitions (optional - auto-discovered by default)
[[templates.definitions]]
id = "home"
path = "templates/home.html"
template_type = "home"

[[templates.definitions]]
id = "single"
path = "templates/single.html"
template_type = "single"

[[templates.definitions]]
id = "page"
path = "templates/page.html"
template_type = "page"

[[templates.definitions]]
id = "archive"
path = "templates/archive.html"
template_type = "archive"

[[templates.definitions]]
id = "search"
path = "templates/search.html"
template_type = "search"

[[templates.definitions]]
id = "404"
path = "templates/404.html"
template_type = "404"

# ============================================
# ASSETS (CSS/JS)
# ============================================
[[assets.css]]
path = "assets/css/main.css"
handle = "theme-styles"               # Unique handle
location = "header"                   # header|footer

[[assets.css]]
path = "assets/css/components.css"
handle = "theme-components"
location = "header"

[[assets.js]]
path = "assets/js/main.js"
handle = "theme-scripts"
location = "footer"                   # JS typically in footer
dependencies = ["alpine"]             # Load after these scripts

[[assets.js]]
path = "assets/js/navigation.js"
handle = "theme-navigation"
location = "footer"

# ============================================
# CUSTOMIZER SETTINGS
# ============================================
[settings.schema.primary_color]
setting_type = "color"
label = "Primary Color"
default = "#007bff"
section = "colors"

[settings.schema.secondary_color]
setting_type = "color"
label = "Secondary Color"
default = "#6c757d"
section = "colors"

[settings.schema.accent_color]
setting_type = "color"
label = "Accent Color"
default = "#fd7e14"
section = "colors"

[settings.schema.logo]
setting_type = "image"
label = "Site Logo"
section = "header"

[settings.schema.logo_height]
setting_type = "number"
label = "Logo Height (px)"
default = 60
section = "header"

[settings.schema.header_style]
setting_type = "select"
label = "Header Style"
options = ["default", "centered", "minimal"]
default = "default"
section = "header"

[settings.schema.footer_text]
setting_type = "text"
label = "Footer Copyright Text"
default = "© 2024 My Site. All rights reserved."
section = "footer"

[settings.schema.show_footer_widgets]
setting_type = "boolean"
label = "Show Footer Widgets"
default = true
section = "footer"

[settings.schema.sidebar_position]
setting_type = "select"
label = "Sidebar Position"
options = ["left", "right", "none"]
default = "right"
section = "layout"

[settings.schema.container_width]
setting_type = "select"
label = "Container Width"
options = ["narrow", "default", "wide", "full"]
default = "default"
section = "layout"

[settings.schema.custom_css]
setting_type = "code"
label = "Custom CSS"
default = ""
section = "advanced"

# ============================================
# COLOR PALETTE
# ============================================
[colors]
primary = "#007bff"
secondary = "#6c757d"
accent = "#fd7e14"
success = "#28a745"
warning = "#ffc107"
danger = "#dc3545"
info = "#17a2b8"
light = "#f8f9fa"
dark = "#343a40"
background = "#ffffff"
text = "#212529"
muted = "#6c757d"
border = "#dee2e6"

# ============================================
# TYPOGRAPHY
# ============================================
[typography]
heading_font = "Poppins"
body_font = "Inter"
code_font = "JetBrains Mono"
base_size = 16
line_height = 1.6
heading_weight = 600
body_weight = 400

# ============================================
# LAYOUT CONFIGURATION
# ============================================
[layout]
container_width = 1200
container_narrow = 720
container_wide = 1400
sidebar_width = 300
header_height = 80
mobile_breakpoint = 768
tablet_breakpoint = 1024
desktop_breakpoint = 1200

# ============================================
# BLOCK PATTERNS
# ============================================
[[patterns]]
name = "hero-banner"
title = "Hero Banner"
categories = ["layout", "featured"]
content = """
<section class="hero">
  <div class="hero-content">
    <h1>Welcome to Our Site</h1>
    <p>Your tagline goes here</p>
    <a href="#" class="btn btn-primary">Get Started</a>
  </div>
</section>
"""

[[patterns]]
name = "cta-section"
title = "Call to Action"
categories = ["layout"]
content = """
<section class="cta">
  <h2>Ready to Get Started?</h2>
  <p>Join thousands of happy customers today.</p>
  <a href="#" class="btn btn-accent">Sign Up Now</a>
</section>
"""

[[patterns]]
name = "feature-grid"
title = "Feature Grid"
categories = ["layout", "columns"]
content = """
<div class="feature-grid">
  <div class="feature">
    <h3>Feature One</h3>
    <p>Description here</p>
  </div>
  <div class="feature">
    <h3>Feature Two</h3>
    <p>Description here</p>
  </div>
  <div class="feature">
    <h3>Feature Three</h3>
    <p>Description here</p>
  </div>
</div>
"""

# ============================================
# STYLE VARIATIONS
# ============================================
[[variations]]
name = "light"
label = "Light Mode"
default = true

[variations.colors]
background = "#ffffff"
text = "#212529"
surface = "#f8f9fa"

[[variations]]
name = "dark"
label = "Dark Mode"

[variations.colors]
background = "#1a1a1a"
text = "#f8f9fa"
surface = "#2d2d2d"

[[variations]]
name = "high-contrast"
label = "High Contrast"

[variations.colors]
background = "#000000"
text = "#ffffff"
primary = "#00ff00"

# ============================================
# FEATURE SUPPORT
# ============================================
[supports]
posts = true
pages = true
widgets = true
menus = true
custom_logo = true
featured_images = true
post_thumbnails = true
comments = true
search = true
custom_background = false
custom_header = true
editor_styles = true
responsive_embeds = true
```

---

## DIRECTORY STRUCTURE

```
my-theme/
├── theme.toml               # REQUIRED: Theme manifest
├── screenshot.png           # REQUIRED: Theme preview (1200x900)
├── templates/               # REQUIRED: Template files
│   ├── base.html            # Base layout (all pages extend this)
│   ├── home.html            # Homepage template
│   ├── page.html            # Single page template
│   ├── single.html          # Single post template
│   ├── archive.html         # Archive/listing template
│   ├── category.html        # Category archive
│   ├── author.html          # Author archive
│   ├── search.html          # Search results
│   └── 404.html             # Not found page
├── parts/                   # Template partials
│   ├── header.html          # Site header
│   ├── footer.html          # Site footer
│   ├── navigation.html      # Main navigation
│   ├── sidebar.html         # Sidebar
│   ├── post-card.html       # Post card component
│   ├── pagination.html      # Pagination component
│   ├── comments.html        # Comments section
│   └── meta.html            # Post meta (date, author, etc.)
├── assets/
│   ├── css/
│   │   ├── variables.css    # CSS custom properties
│   │   ├── base.css         # Reset and base styles
│   │   ├── typography.css   # Typography styles
│   │   ├── components.css   # UI components
│   │   ├── layout.css       # Layout utilities
│   │   ├── responsive.css   # Media queries
│   │   └── main.css         # Main entry (imports all)
│   ├── js/
│   │   ├── main.js          # Main JavaScript
│   │   ├── navigation.js    # Mobile nav toggle
│   │   └── theme-toggle.js  # Dark/light mode
│   ├── images/
│   │   ├── logo.svg
│   │   └── default-featured.jpg
│   └── fonts/               # Custom fonts (if any)
└── languages/               # Translation files
    ├── en_US.po
    └── es_ES.po
```

---

## TEMPLATE HIERARCHY

RustPress resolves templates in a specific order. The first matching template is used.

### Homepage
1. `home.html`
2. `index.html`

### Single Post
1. `single-{post_type}-{slug}.html` (e.g., `single-post-hello-world.html`)
2. `single-{post_type}.html` (e.g., `single-post.html`)
3. `single.html`
4. `singular.html`
5. `index.html`

### Single Page
1. `page-{slug}.html` (e.g., `page-about.html`)
2. `page-{id}.html` (e.g., `page-42.html`)
3. `page.html`
4. `singular.html`
5. `index.html`

### Category Archive
1. `category-{slug}.html` (e.g., `category-news.html`)
2. `category-{id}.html`
3. `category.html`
4. `archive.html`
5. `index.html`

### Tag Archive
1. `tag-{slug}.html`
2. `tag-{id}.html`
3. `tag.html`
4. `archive.html`
5. `index.html`

### Author Archive
1. `author-{slug}.html` (e.g., `author-john.html`)
2. `author-{id}.html`
3. `author.html`
4. `archive.html`
5. `index.html`

### Date Archive
1. `date.html`
2. `archive.html`
3. `index.html`

### Search Results
1. `search.html`
2. `index.html`

### 404 Not Found
1. `404.html`
2. `index.html`

### Custom Post Type Archive
1. `archive-{post_type}.html`
2. `archive.html`
3. `index.html`

---

## TEMPLATE SYNTAX (Tera/Jinja2)

### Variables
```html
{{ variable }}
{{ object.property }}
{{ array[0] }}
{{ nested.deep.value }}
```

### Output with Escaping
```html
{{ user_input }}                      <!-- Auto-escaped (safe) -->
{{ html_content | safe }}             <!-- Raw HTML (trust source!) -->
```

### Conditionals
```html
{% if condition %}
  <p>Condition is true</p>
{% elif other_condition %}
  <p>Other condition is true</p>
{% else %}
  <p>No conditions met</p>
{% endif %}

<!-- Boolean checks -->
{% if user.authenticated %}
{% if post.featured_image %}
{% if posts | length > 0 %}

<!-- Comparison operators -->
{% if count > 10 %}
{% if status == "published" %}
{% if name != "admin" %}

<!-- Logical operators -->
{% if user.authenticated and user.is_admin %}
{% if is_home or is_archive %}
{% if not is_404 %}
```

### Loops
```html
{% for post in posts %}
  <article>
    <h2>{{ post.title }}</h2>
    <p>{{ post.excerpt }}</p>
  </article>
{% empty %}
  <p>No posts found.</p>
{% endfor %}

<!-- Loop variables -->
{% for item in items %}
  {{ loop.index }}      <!-- 1-based index -->
  {{ loop.index0 }}     <!-- 0-based index -->
  {{ loop.first }}      <!-- true if first iteration -->
  {{ loop.last }}       <!-- true if last iteration -->
  {{ loop.length }}     <!-- total items -->
{% endfor %}
```

### Template Inheritance
```html
{# base.html - Parent template #}
<!DOCTYPE html>
<html lang="{{ site.language | default(value='en') }}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}{{ site.name }}{% endblock %}</title>
  {% block head %}{% endblock %}
</head>
<body class="{% block body_class %}{% endblock %}">
  {% include "parts/header.html" %}

  <main>
    {% block content %}{% endblock %}
  </main>

  {% include "parts/footer.html" %}

  {% block scripts %}{% endblock %}
</body>
</html>

{# page.html - Child template #}
{% extends "base.html" %}

{% block title %}{{ page.title }} - {{ site.name }}{% endblock %}

{% block body_class %}page page-{{ page.slug }}{% endblock %}

{% block content %}
<article class="page">
  <header>
    <h1>{{ page.title }}</h1>
  </header>
  <div class="page-content">
    {{ page.content | safe }}
  </div>
</article>
{% endblock %}
```

### Including Partials
```html
{% include "parts/header.html" %}
{% include "parts/footer.html" %}
{% include "parts/sidebar.html" %}

<!-- With variables -->
{% include "parts/post-card.html" %}
<!-- post variable is automatically available -->

<!-- Conditional include -->
{% if settings.show_sidebar %}
  {% include "parts/sidebar.html" %}
{% endif %}
```

### Macros (Reusable Components)
```html
{# Define macro #}
{% macro button(text, url, style="primary") %}
<a href="{{ url }}" class="btn btn-{{ style }}">{{ text }}</a>
{% endmacro %}

{# Use macro #}
{{ self::button(text="Learn More", url="/about") }}
{{ self::button(text="Contact", url="/contact", style="secondary") }}

{# Card macro #}
{% macro card(title, content, image="") %}
<div class="card">
  {% if image %}
    <img src="{{ image }}" alt="{{ title }}" class="card-image">
  {% endif %}
  <div class="card-body">
    <h3 class="card-title">{{ title }}</h3>
    <p class="card-content">{{ content }}</p>
  </div>
</div>
{% endmacro %}

{{ self::card(title="Hello", content="World", image="/images/hero.jpg") }}
```

### Setting Variables
```html
{% set page_title = post.title ~ " - " ~ site.name %}
{% set show_sidebar = settings.sidebar_position != "none" %}
{% set posts_per_row = 3 %}
```

---

## COMMON FILTERS

| Filter | Example | Description |
|--------|---------|-------------|
| `safe` | `{{ html \| safe }}` | Output raw HTML (no escaping) |
| `escape` | `{{ text \| escape }}` | HTML escape (default) |
| `upper` | `{{ text \| upper }}` | UPPERCASE |
| `lower` | `{{ text \| lower }}` | lowercase |
| `title` | `{{ text \| title }}` | Title Case |
| `capitalize` | `{{ text \| capitalize }}` | First letter uppercase |
| `trim` | `{{ text \| trim }}` | Remove whitespace |
| `truncate` | `{{ text \| truncate(length=100) }}` | Truncate with ellipsis |
| `wordcount` | `{{ text \| wordcount }}` | Count words |
| `striptags` | `{{ html \| striptags }}` | Remove HTML tags |
| `date` | `{{ date \| date(format="%Y-%m-%d") }}` | Format date |
| `default` | `{{ val \| default(value="N/A") }}` | Default if empty |
| `join` | `{{ list \| join(sep=", ") }}` | Join array |
| `split` | `{{ text \| split(pat=",") }}` | Split string |
| `length` | `{{ list \| length }}` | Array/string length |
| `first` | `{{ list \| first }}` | First element |
| `last` | `{{ list \| last }}` | Last element |
| `nth` | `{{ list \| nth(n=2) }}` | Nth element |
| `sort` | `{{ list \| sort(attribute="name") }}` | Sort array |
| `reverse` | `{{ list \| reverse }}` | Reverse array |
| `unique` | `{{ list \| unique }}` | Remove duplicates |
| `slice` | `{{ list \| slice(start=0, end=5) }}` | Slice array |
| `json_encode` | `{{ obj \| json_encode }}` | JSON stringify |
| `filesizeformat` | `{{ bytes \| filesizeformat }}` | Human file size |
| `urlencode` | `{{ url \| urlencode }}` | URL encode |
| `replace` | `{{ text \| replace(from="a", to="b") }}` | Replace text |

### Date Format Tokens
```
%Y - Year (2024)
%m - Month (01-12)
%d - Day (01-31)
%H - Hour 24h (00-23)
%M - Minute (00-59)
%S - Second (00-59)
%B - Month name (January)
%b - Month abbr (Jan)
%A - Day name (Monday)
%a - Day abbr (Mon)
```

Example:
```html
{{ post.date | date(format="%B %d, %Y") }}     <!-- January 15, 2024 -->
{{ post.date | date(format="%Y-%m-%d") }}      <!-- 2024-01-15 -->
{{ post.date | date(format="%b %d, %Y at %H:%M") }}  <!-- Jan 15, 2024 at 14:30 -->
```

---

## TEMPLATE CONTEXT VARIABLES

### Global Variables (Available Everywhere)
```html
{{ site.name }}                <!-- Site name -->
{{ site.url }}                 <!-- Site URL -->
{{ site.description }}         <!-- Site tagline -->
{{ site.language }}            <!-- Site language (en_US) -->

{{ current_user }}             <!-- Current logged-in user (or null) -->
{{ current_user.id }}
{{ current_user.name }}
{{ current_user.email }}
{{ current_user.avatar }}
{{ current_user.is_admin }}

{{ settings.primary_color }}   <!-- Theme customizer settings -->
{{ settings.logo }}
{{ settings.footer_text }}

{{ theme.name }}               <!-- Current theme info -->
{{ theme.version }}
{{ theme.url }}

<!-- Page type booleans -->
{{ is_home }}                  <!-- Is homepage -->
{{ is_front_page }}            <!-- Is static front page -->
{{ is_single }}                <!-- Is single post -->
{{ is_page }}                  <!-- Is single page -->
{{ is_archive }}               <!-- Is any archive -->
{{ is_category }}              <!-- Is category archive -->
{{ is_tag }}                   <!-- Is tag archive -->
{{ is_author }}                <!-- Is author archive -->
{{ is_date }}                  <!-- Is date archive -->
{{ is_search }}                <!-- Is search results -->
{{ is_404 }}                   <!-- Is 404 page -->
```

### Single Post/Page Variables
```html
{{ post.id }}
{{ post.title }}
{{ post.slug }}
{{ post.content }}             <!-- Full HTML content -->
{{ post.excerpt }}             <!-- Auto or manual excerpt -->
{{ post.date }}                <!-- Publish date -->
{{ post.modified }}            <!-- Last modified date -->
{{ post.status }}              <!-- draft|published|archived -->
{{ post.type }}                <!-- post|page|custom -->
{{ post.format }}              <!-- standard|video|gallery|etc -->

{{ post.author }}              <!-- Author object -->
{{ post.author.id }}
{{ post.author.name }}
{{ post.author.slug }}
{{ post.author.bio }}
{{ post.author.avatar }}
{{ post.author.url }}

{{ post.featured_image }}      <!-- Featured image object -->
{{ post.featured_image.url }}
{{ post.featured_image.alt }}
{{ post.featured_image.width }}
{{ post.featured_image.height }}
{{ post.featured_image.sizes.thumbnail }}
{{ post.featured_image.sizes.medium }}
{{ post.featured_image.sizes.large }}

{{ post.categories }}          <!-- Array of categories -->
{{ post.tags }}                <!-- Array of tags -->
{{ post.comments_count }}
{{ post.reading_time }}        <!-- Estimated reading time -->

{{ post.meta.custom_field }}   <!-- Custom fields -->
{{ post.seo.title }}           <!-- SEO title -->
{{ post.seo.description }}     <!-- SEO description -->
```

### Archive Variables
```html
{{ posts }}                    <!-- Array of posts -->
{{ posts | length }}

{{ pagination.current }}       <!-- Current page number -->
{{ pagination.total }}         <!-- Total pages -->
{{ pagination.total_items }}   <!-- Total posts -->
{{ pagination.per_page }}      <!-- Posts per page -->
{{ pagination.prev_url }}      <!-- Previous page URL (or null) -->
{{ pagination.next_url }}      <!-- Next page URL (or null) -->
{{ pagination.pages }}         <!-- Array of page numbers -->

<!-- For category/tag archives -->
{{ term.id }}
{{ term.name }}
{{ term.slug }}
{{ term.description }}
{{ term.count }}               <!-- Number of posts -->

<!-- For author archives -->
{{ author.id }}
{{ author.name }}
{{ author.slug }}
{{ author.bio }}
{{ author.avatar }}
```

### Search Variables
```html
{{ search_query }}             <!-- Search query string -->
{{ search_results }}           <!-- Array of matching posts -->
{{ search_results | length }}  <!-- Number of results -->
```

### Menu Variables
```html
{{ menus.primary }}            <!-- Primary menu -->
{{ menus.footer }}             <!-- Footer menu -->
{{ menus.social }}             <!-- Social menu -->

{% for item in menus.primary %}
  {{ item.title }}
  {{ item.url }}
  {{ item.target }}            <!-- _blank, etc -->
  {{ item.classes }}           <!-- CSS classes -->
  {{ item.current }}           <!-- true if current page -->
  {{ item.children }}          <!-- Submenu items -->
{% endfor %}
```

### Widget Areas
```html
{{ widgets.sidebar }}          <!-- Sidebar widgets -->
{{ widgets.footer_1 }}         <!-- Footer column 1 -->
{{ widgets.footer_2 }}
{{ widgets.footer_3 }}

{% for widget in widgets.sidebar %}
  {{ widget.title }}
  {{ widget.content | safe }}
{% endfor %}
```

---

## CSS DESIGN TOKENS (VARIABLES)

### Recommended CSS Variables Structure
```css
:root {
  /* ==================== Colors ==================== */
  --color-primary: #007bff;
  --color-primary-hover: #0056b3;
  --color-primary-light: #e7f1ff;

  --color-secondary: #6c757d;
  --color-secondary-hover: #545b62;

  --color-accent: #fd7e14;
  --color-accent-hover: #e96b02;

  --color-success: #28a745;
  --color-warning: #ffc107;
  --color-danger: #dc3545;
  --color-info: #17a2b8;

  --color-background: #ffffff;
  --color-surface: #f8f9fa;
  --color-text: #212529;
  --color-text-muted: #6c757d;
  --color-text-light: #adb5bd;
  --color-border: #dee2e6;
  --color-border-light: #e9ecef;

  /* ==================== Typography ==================== */
  --font-family-heading: 'Poppins', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-family-body: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-family-mono: 'JetBrains Mono', 'Fira Code', monospace;

  --font-size-xs: 0.75rem;     /* 12px */
  --font-size-sm: 0.875rem;    /* 14px */
  --font-size-base: 1rem;      /* 16px */
  --font-size-lg: 1.125rem;    /* 18px */
  --font-size-xl: 1.25rem;     /* 20px */
  --font-size-2xl: 1.5rem;     /* 24px */
  --font-size-3xl: 1.875rem;   /* 30px */
  --font-size-4xl: 2.25rem;    /* 36px */
  --font-size-5xl: 3rem;       /* 48px */

  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;

  --line-height-tight: 1.25;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.75;

  /* ==================== Spacing ==================== */
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-5: 1.25rem;   /* 20px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-10: 2.5rem;   /* 40px */
  --space-12: 3rem;     /* 48px */
  --space-16: 4rem;     /* 64px */
  --space-20: 5rem;     /* 80px */

  /* ==================== Layout ==================== */
  --container-sm: 640px;
  --container-md: 768px;
  --container-lg: 1024px;
  --container-xl: 1200px;
  --container-2xl: 1400px;

  --sidebar-width: 300px;
  --header-height: 80px;

  /* ==================== Borders ==================== */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-full: 9999px;

  --border-width: 1px;
  --border-width-2: 2px;

  /* ==================== Shadows ==================== */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.07), 0 2px 4px rgba(0, 0, 0, 0.05);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1), 0 4px 6px rgba(0, 0, 0, 0.05);
  --shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.1), 0 10px 10px rgba(0, 0, 0, 0.04);

  /* ==================== Transitions ==================== */
  --transition-fast: 150ms ease;
  --transition-normal: 250ms ease;
  --transition-slow: 350ms ease;

  /* ==================== Z-Index ==================== */
  --z-dropdown: 100;
  --z-sticky: 200;
  --z-fixed: 300;
  --z-modal-backdrop: 400;
  --z-modal: 500;
  --z-popover: 600;
  --z-tooltip: 700;
}

/* ==================== Dark Mode ==================== */
[data-theme="dark"],
.dark-mode {
  --color-background: #1a1a1a;
  --color-surface: #2d2d2d;
  --color-text: #f8f9fa;
  --color-text-muted: #adb5bd;
  --color-border: #404040;
  --color-border-light: #333333;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.5);
}
```

---

## RESPONSIVE BREAKPOINTS

### Mobile-First Approach (Recommended)
```css
/* Base styles (mobile) */
.container {
  width: 100%;
  padding: 0 var(--space-4);
}

/* Tablet (768px+) */
@media (min-width: 768px) {
  .container {
    max-width: var(--container-md);
    margin: 0 auto;
  }
}

/* Desktop (1024px+) */
@media (min-width: 1024px) {
  .container {
    max-width: var(--container-lg);
  }
}

/* Large Desktop (1200px+) */
@media (min-width: 1200px) {
  .container {
    max-width: var(--container-xl);
  }
}
```

### Common Breakpoint Values
```css
/* Mobile:  < 768px  (base styles) */
/* Tablet:  >= 768px */
/* Desktop: >= 1024px */
/* Large:   >= 1200px */
/* XLarge:  >= 1400px */
```

---

## EXAMPLE TEMPLATES

### base.html
```html
<!DOCTYPE html>
<html lang="{{ site.language | default(value='en') }}" data-theme="light">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}{{ site.name }}{% endblock %}</title>

  {% if post.seo.description %}
    <meta name="description" content="{{ post.seo.description }}">
  {% else %}
    <meta name="description" content="{{ site.description }}">
  {% endif %}

  <link rel="stylesheet" href="{{ theme.url }}/assets/css/main.css">
  {% block head %}{% endblock %}
</head>
<body class="{% block body_class %}{% endblock %}">
  {% include "parts/header.html" %}

  <main id="main-content" class="site-main">
    {% block content %}{% endblock %}
  </main>

  {% include "parts/footer.html" %}

  <script src="{{ theme.url }}/assets/js/main.js" defer></script>
  {% block scripts %}{% endblock %}
</body>
</html>
```

### single.html
```html
{% extends "base.html" %}

{% block title %}{{ post.title }} - {{ site.name }}{% endblock %}

{% block body_class %}single single-post{% endblock %}

{% block content %}
<article class="post" id="post-{{ post.id }}">
  <header class="post-header">
    {% if post.featured_image %}
      <img
        src="{{ post.featured_image.url }}"
        alt="{{ post.featured_image.alt | default(value=post.title) }}"
        class="post-featured-image"
      >
    {% endif %}

    <h1 class="post-title">{{ post.title }}</h1>

    <div class="post-meta">
      <span class="post-date">
        {{ post.date | date(format="%B %d, %Y") }}
      </span>
      <span class="post-author">
        by <a href="{{ post.author.url }}">{{ post.author.name }}</a>
      </span>
      {% if post.categories | length > 0 %}
        <span class="post-categories">
          in
          {% for cat in post.categories %}
            <a href="{{ cat.url }}">{{ cat.name }}</a>{% if not loop.last %}, {% endif %}
          {% endfor %}
        </span>
      {% endif %}
    </div>
  </header>

  <div class="post-content">
    {{ post.content | safe }}
  </div>

  {% if post.tags | length > 0 %}
    <footer class="post-footer">
      <div class="post-tags">
        {% for tag in post.tags %}
          <a href="{{ tag.url }}" class="tag">{{ tag.name }}</a>
        {% endfor %}
      </div>
    </footer>
  {% endif %}
</article>

{% if settings.show_comments %}
  {% include "parts/comments.html" %}
{% endif %}
{% endblock %}
```

### archive.html
```html
{% extends "base.html" %}

{% block title %}
  {% if is_category %}
    {{ term.name }} - {{ site.name }}
  {% elif is_author %}
    Posts by {{ author.name }} - {{ site.name }}
  {% elif is_search %}
    Search: {{ search_query }} - {{ site.name }}
  {% else %}
    Archive - {{ site.name }}
  {% endif %}
{% endblock %}

{% block body_class %}archive{% endblock %}

{% block content %}
<div class="archive-header">
  {% if is_category %}
    <h1>Category: {{ term.name }}</h1>
    {% if term.description %}
      <p class="term-description">{{ term.description }}</p>
    {% endif %}
  {% elif is_author %}
    <h1>Posts by {{ author.name }}</h1>
    {% if author.bio %}
      <p class="author-bio">{{ author.bio }}</p>
    {% endif %}
  {% elif is_search %}
    <h1>Search Results for "{{ search_query }}"</h1>
    <p>{{ posts | length }} results found</p>
  {% else %}
    <h1>All Posts</h1>
  {% endif %}
</div>

<div class="posts-grid">
  {% for post in posts %}
    {% include "parts/post-card.html" %}
  {% empty %}
    <p class="no-posts">No posts found.</p>
  {% endfor %}
</div>

{% if pagination.total > 1 %}
  {% include "parts/pagination.html" %}
{% endif %}
{% endblock %}
```

### parts/post-card.html
```html
<article class="post-card">
  {% if post.featured_image %}
    <a href="/post/{{ post.slug }}" class="post-card-image">
      <img
        src="{{ post.featured_image.sizes.medium }}"
        alt="{{ post.featured_image.alt | default(value=post.title) }}"
        loading="lazy"
      >
    </a>
  {% endif %}

  <div class="post-card-content">
    <h2 class="post-card-title">
      <a href="/post/{{ post.slug }}">{{ post.title }}</a>
    </h2>

    <div class="post-card-meta">
      <time datetime="{{ post.date | date(format='%Y-%m-%d') }}">
        {{ post.date | date(format="%b %d, %Y") }}
      </time>
    </div>

    {% if post.excerpt %}
      <p class="post-card-excerpt">{{ post.excerpt | truncate(length=150) }}</p>
    {% endif %}

    <a href="/post/{{ post.slug }}" class="post-card-link">Read More</a>
  </div>
</article>
```

### parts/pagination.html
```html
<nav class="pagination" aria-label="Pagination">
  {% if pagination.prev_url %}
    <a href="{{ pagination.prev_url }}" class="pagination-prev">
      &larr; Previous
    </a>
  {% endif %}

  <span class="pagination-info">
    Page {{ pagination.current }} of {{ pagination.total }}
  </span>

  {% if pagination.next_url %}
    <a href="{{ pagination.next_url }}" class="pagination-next">
      Next &rarr;
    </a>
  {% endif %}
</nav>
```

---

## BEST PRACTICES

1. **Follow template hierarchy** - Use the standard template naming convention
2. **Use template parts** - Extract reusable components (header, footer, cards)
3. **Organize assets by type** - Keep css/, js/, images/ separate
4. **Use CSS variables** - Enable easy theming and customization
5. **Mobile-first design** - Start with mobile styles, enhance for larger screens
6. **Provide customizer options** - Let users adjust colors, fonts, layout
7. **Optimize images** - Use WebP format, provide srcset for responsive images
8. **Minimize CSS/JS** - Concatenate and minify for production
9. **Follow WCAG guidelines** - Ensure accessibility (contrast, focus states, ARIA)
10. **Document features** - Include README with theme capabilities
11. **Escape user content** - Only use `| safe` for trusted HTML content
12. **Use semantic HTML** - article, nav, main, aside, header, footer
13. **Support dark mode** - Provide light/dark variations
14. **Test across browsers** - Chrome, Firefox, Safari, Edge
