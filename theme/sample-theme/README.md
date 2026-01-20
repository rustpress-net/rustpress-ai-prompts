# Starter Theme

A minimal RustPress theme example demonstrating core concepts.

## Features

- **Clean Design**: Minimalist, responsive layout
- **Template Hierarchy**: home, single, page, archive, 404
- **Template Parts**: Reusable header, footer, pagination
- **Customizer**: Primary color, logo settings
- **CSS Variables**: Easy theming

## Templates

| Template | Purpose |
|----------|---------|
| `base.html` | Base layout (extended by all) |
| `home.html` | Homepage with post list |
| `single.html` | Single post display |
| `page.html` | Single page display |
| `archive.html` | Category/author archives |
| `404.html` | Not found page |

## Customization

Edit CSS variables in `assets/css/style.css`:

```css
:root {
    --color-primary: #3b82f6;
    --color-text: #1f2937;
    --container-width: 800px;
}
```

## File Structure

```
starter/
├── theme.toml
├── templates/
│   ├── base.html
│   ├── home.html
│   ├── single.html
│   ├── page.html
│   ├── archive.html
│   └── 404.html
├── parts/
│   ├── header.html
│   ├── footer.html
│   └── pagination.html
└── assets/
    ├── css/style.css
    └── js/main.js
```
