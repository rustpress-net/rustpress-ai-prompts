# rustpress-ai-prompts — AI Context

> **Purpose**: Orient an AI agent to this repo without reading the whole tree. Pair with the RustPress organisation context in `rustpress-core-base/.ai/context/CONTEXT_BASE.md`.

## Project

`rustpress-ai-prompts` is a **library of AI prompt references** for building RustPress plugins, themes, apps, and functions. Each prompt is a self-contained Markdown reference designed to be pasted into a Claude/Cursor/Copilot context window so the AI can generate compatible RustPress code without having read the whole core codebase. Each category also ships **working sample projects** for AI to crib from.

This repo is one of the **launch differentiators** for RustPress v1.0 — it powers the "build a plugin with AI in an hour" narrative. The audit calls it polished and launch-ready as-is.

## Tech stack

- **Format**: Markdown (no code build)
- **Language of samples**: Rust (plugins, apps, functions) + Tera HTML (themes)
- **License**: MIT
- **Tooling**: none required to use. Sample projects are real `Cargo.toml`-based crates and `theme.json`-based themes that compile/render on a normal RustPress install.

## Directory layout

```
rustpress-ai-prompts/
├── README.md
├── plugin/
│   ├── PROMPT.md            # ~810 lines: manifest, lifecycle, API, settings, migrations
│   ├── sample-plugin/       # Hello World plugin (API endpoint + shortcode)
│   ├── advanced-plugin/     # richer example
│   └── auth-plugin/         # restructured authentication example
├── theme/
│   ├── PROMPT.md            # ~1,178 lines: templates, hierarchy, Tera syntax, CSS tokens
│   ├── sample-theme/        # minimal starter theme
│   └── advanced-theme/
├── app/
│   ├── PROMPT.md            # ~1,195 lines: router, handlers, extractors, middleware, DB
│   ├── sample-app/          # Todo CRUD API with Axum
│   └── advanced-app/
└── function/
    ├── PROMPT.md            # ~1,101 lines: hooks, actions, filters, lifecycle, utilities
    ├── sample-function/
    └── advanced-function/
```

## Public API / what this repo exposes

Four canonical prompt categories, each delivering one `PROMPT.md` plus one or more sample projects:

| Category | PROMPT.md lines | Use case |
|----------|----------------:|----------|
| **plugin** | ~810 | Manifest schema, lifecycle, API, database, settings, blocks, CLI, cron, WASM, feature flags |
| **theme** | ~1,178 | Manifest, templates, template hierarchy, Tera syntax, CSS variables, customizer, hooks, responsive design |
| **app** | ~1,195 | Router, handlers, extractors, middleware, database, error handling |
| **function** | ~1,101 | Hooks, actions, filters, lifecycle, utility helpers |

Usage pattern (from README):

1. Copy the relevant `PROMPT.md` into the AI assistant's context window.
2. Reference the sibling `sample-*` project for working implementation patterns.
3. Build a RustPress component following the prompt's spec.

## How to build / test

Nothing builds here — the prompts are docs and the samples are reference snippets. To smoke-test a sample:

```bash
# Plugin / app / function samples (Rust):
cd plugin/sample-plugin && cargo build
cd app/sample-app       && cargo build && cargo run

# Theme sample:
cd theme/sample-theme
# Drop into a running RustPress install's themes/ dir to render.
```

Quality bar for every sample: must `cargo build` clean on the version of RustPress declared in its `Cargo.toml`. Broken samples actively mislead the AI that copies them.

## Cross-repo dependencies

- **Depends on**: `rustpress-core-base` (the runtime that plugins/themes/apps target) **conceptually**. Samples reference `rustpress-sdk` (Rust) or the core crates by version, not by path. The prompts assume RustPress ≥1.0.
- **Depended on by**: `rustpress-web-dev` (linked from the docs site) and `rustpress.net` marketing. Users discover this repo via launch announcements; tooling (Cursor/Claude users) may add it as a git submodule or context source.

## Conventions

- **License**: MIT (stated in README; align to `MIT OR Apache-2.0` to match org if/when convenient — README currently says MIT only)
- **Commits**: Conventional Commits — recent example: `Restructure authentication as standalone plugin`
- **Prompt format**: Markdown only, ATX headings, fenced code blocks with explicit language tag (` ```rust `, ` ```html `, ` ```toml `, ` ```json `). Tables for spec data. Examples are runnable, not pseudocode.
- **Sample naming**: every category has `sample-*` (minimal) and `advanced-*` (broader). Don't break this convention — the prompts cross-reference it.
- **Sample contracts**: each sample's `Cargo.toml` / `theme.json` declares a `rustpress` minimum version. Bump this in lockstep with the org's actual release tier.

## Status

- Release readiness: **🟢 READY** for v1.0 (see `AUDIT-docs-prompts.md` PART 2 and master `AUDIT.md`)
- Audit verdict: "**LAUNCH IMMEDIATELY** — Polished, comprehensive, and directly differentiates RustPress as AI-first."
- No P0/P1 blockers identified.
- Should be **prominently featured** in the v1.0 launch story.

## Known issues / TODOs

- **None blocking**. Per audit: prompts complete, samples work, README clear, license stated.
- **Nice-to-have**: pin every sample's `rustpress` minimum version to the actual GA value (today some samples reference older versions).
- **Nice-to-have**: consider adding a `cli/` category (prompt for building CLI extensions to the `rustpress` binary).
- **Nice-to-have**: a top-level "how to feed these to your AI tool" guide for popular tools (Claude Code, Cursor, GitHub Copilot Chat, Continue) — examples of `.cursorrules`, `CLAUDE.md`, etc.

## When working in this repo

- The audience is **AI tooling**, not humans. Optimise for context-window efficiency: no marketing copy, no decorative emoji, complete code blocks (no `// ... rest of impl` ellipses).
- Every code example must compile / render. Run `cargo build` on Rust samples before committing. Broken examples poison every downstream generation.
- When the core RustPress API changes (new public symbols, renames in `rustpress-sdk-*`, changes to `plugin.toml` schema), the corresponding `PROMPT.md` and `sample-*` need updates **in the same release cycle**. Out-of-sync prompts are worse than missing prompts.
- Avoid copy-pasting the same definition across prompts (e.g. `RustPressContext` appears in all four PROMPT.md files). When something changes, grep across all four and update consistently.
- Keep PROMPT.md files self-contained. Cross-linking between prompts is acceptable for "see also" but a prompt should make sense on its own when pasted into a context window.
