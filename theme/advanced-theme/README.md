# Business Starter Theme

A professional, feature-rich RustPress theme demonstrating advanced theme development patterns.

## Features

- **Dark Mode**: Automatic or manual dark/light theme switching
- **Customizer Integration**: Extensive customization options for colors, typography, layout
- **Block Patterns**: Pre-designed content sections for quick page building
- **Responsive Design**: Mobile-first approach with fluid layouts
- **Accessibility**: WCAG compliant with keyboard navigation and screen reader support
- **Performance**: Optimized CSS and lazy-loading images

## Architecture

```
advanced-theme/
├── theme.toml              # Theme manifest with all configuration
├── templates/              # Tera/Jinja2 templates
│   ├── base.html           # Base layout
│   ├── index.html          # Homepage/blog listing
│   ├── single.html         # Single post
│   ├── page.html           # Static page
│   ├── partials/           # Reusable components
│   │   ├── header.html
│   │   └── footer.html
│   └── blocks/             # Block templates
├── assets/
│   ├── css/
│   │   ├── main.css        # Main stylesheet
│   │   └── dark-mode.css   # Dark mode overrides
│   └── js/
│       ├── theme.js        # Theme functionality
│       └── navigation.js   # Menu handling
├── patterns/               # Block patterns
│   ├── hero-cta.html
│   └── features-grid.html
└── src/                    # Rust functions (optional)
```

## Key Patterns Demonstrated

### 1. Theme Manifest (theme.toml)
- Feature support declarations
- Image size definitions
- Menu and widget area registration
- Customizer panels, sections, and settings
- Asset management
- Block pattern registration

### 2. Customizer Integration
- Color scheme selection (light/dark/auto)
- Custom colors with CSS variables
- Typography options (fonts, sizes)
- Layout controls (container width, sidebar)
- Header and footer options

### 3. Template Hierarchy
- Base template with blocks for extensibility
- Partials for reusable components
- Template inheritance with Tera

### 4. Dark Mode Implementation
- CSS custom properties for theming
- JavaScript toggle with localStorage
- System preference detection (prefers-color-scheme)
- Smooth transitions

### 5. Block Patterns
- Gutenberg-compatible pattern format
- Reusable content sections
- Pattern categories

## Customizer Options

### Colors
- **Color Scheme**: Light, Dark, or Auto (system)
- **Primary Color**: Main accent color
- **Secondary Color**: Supporting color
- **Accent Color**: Highlights and CTAs

### Typography
- **Heading Font**: Choice of 5 fonts
- **Body Font**: Choice of 5 fonts
- **Base Font Size**: 14-20px

### Layout
- **Container Width**: 960-1600px
- **Sidebar Position**: Left, Right, or None

### Header
- **Header Layout**: Standard, Centered, Stacked, Transparent
- **Sticky Header**: Enable/disable
- **Show Search**: Enable/disable

### Footer
- **Footer Layout**: 1-4 columns
- **Copyright Text**: Custom text
- **Back to Top**: Enable/disable

## Template Tags

Available in templates:

```twig
{# Site data #}
{{ site.name }}
{{ site.tagline }}
{{ site.logo }}

{# Theme settings from customizer #}
{{ theme.primary_color }}
{{ theme.heading_font }}
{{ theme.sidebar_position }}

{# Menus #}
{{ menu("primary", class="nav-menu") }}

{# Widgets #}
{{ widgets("sidebar") }}

{# Assets #}
{{ enqueue_styles() }}
{{ enqueue_scripts() }}

{# Utilities #}
{{ post.content | reading_time }}
{{ image | image_url(size='card') }}
```

## CSS Architecture

The theme uses CSS custom properties for theming:

```css
:root {
    --color-primary: #3b82f6;
    --color-bg: #ffffff;
    --color-text: #1e293b;
    /* ... */
}

[data-theme="dark"] {
    --color-bg: #0f172a;
    --color-text: #f1f5f9;
    /* ... */
}
```

## JavaScript API

```javascript
// Theme toggle
window.BusinessStarterTheme.setTheme('dark');

// Navigation
window.BusinessStarterNavigation.closeAllDropdowns();
```

## License

MIT
