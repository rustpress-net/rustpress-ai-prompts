# RustPress AI Prompts

Comprehensive AI prompt references for building RustPress plugins, themes, apps, and functions.

## Structure

```
rustpress-ai-prompts/
├── plugin/
│   ├── PROMPT.md              # Plugin development reference
│   └── sample-plugin/         # Hello World plugin example
├── app/
│   ├── PROMPT.md              # App/routing development reference
│   └── sample-app/            # Todo API example
├── theme/
│   ├── PROMPT.md              # Theme development reference
│   └── sample-theme/          # Starter theme example
└── function/
    ├── PROMPT.md              # Hooks & functions reference
    └── sample-function/       # Hooks implementation example
```

## Prompts

Each `PROMPT.md` file is a self-contained reference for AI assistants to build compatible RustPress components without needing to read the entire codebase.

| Prompt | Lines | Coverage |
|--------|-------|----------|
| Plugin | ~810 | Manifest, lifecycle, API, settings, migrations |
| App | ~1,195 | Router, handlers, extractors, middleware, database |
| Theme | ~1,178 | Templates, hierarchy, Tera syntax, CSS tokens |
| Function | ~1,101 | Hooks, actions, filters, lifecycle, utilities |

## Sample Projects

Each folder includes a minimal working example:

- **sample-plugin**: Hello World plugin with API endpoint and shortcode
- **sample-app**: Todo CRUD API with Axum routing
- **sample-theme**: Minimal starter theme with templates
- **sample-function**: Hook registry with action/filter examples

## Usage

1. Copy the relevant `PROMPT.md` to your AI assistant context
2. Reference the sample project for implementation patterns
3. Build your RustPress component following the specification

## License

MIT
