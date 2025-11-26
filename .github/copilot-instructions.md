<!-- AI Agent Coding Guide: my-http-server (2025.11) -->

# AI Coding Agent Instructions

## Architecture Overview

Actix-web server serving static files and dynamic Markdown via `markdown_ppp` AST + Handlebars. Key flows:

- `.md` files: read → parse (AST) → HTML → inject into `meta/html-t.hbs` → response
- Other files: streamed via `actix_files::NamedFile`
- Missing files: serve `meta/404.html` if present, else plain 404

## Key Modules & Patterns

- **HTTP Routing:** `src/request.rs` (main routes), `src/main.rs` (server setup)
- **Markdown Pipeline:** `src/parser/mod.rs::md2html` (orchestrates parsing & templating)
- **TOC:** `src/parser/markdown.rs::get_toc` (walks files, builds TOC)
- **Templating:** `src/parser/templating.rs` (Handlebars engine, context DSL)
- **Config:** `src/cofg/config.rs` (`OnceCell<RwLock<Cofg>` for global config)
- **Errors:** `src/error.rs` (`AppError` with `Responder` impl, use `?` for propagation)

## Routing & Behavior

- `GET /` → serve `public/index.html` if exists, else TOC via `get_toc` + `md2html`
- `GET /{filename:.*}` → resolve under `public_path`; `.md` → parse/render; dir → TOC; else static file

## Middleware (toggle via `cofg.yaml`)

Chain: Rate limiting → Logger → NormalizePath → Compress → BasicAuth → IP Filter → Routes
Logger uses `%{url}xi` (percent-decoded, leading `/` trimmed)

## Configuration & Templating

- Context DSL: `templating.value: ["name:value"]` (type inference, env support)
- Hot reload: `templating.hot_reload=true` (dev only, rebuilds engine per request)
- Built-in context: `server-version`, `body`, `path`
- CLI overrides: `cofg::build_config_from_cli` (TLS only if cert+key provided)

## Developer Workflow

- **First run:** `cargo run` (writes defaults, exits); run again to start server
- **Tests:** `cargo test` (unit: `src/test/*.rs`, integration: middleware/auth/IP)
- **Lint:** `cargo clippy -- -D warnings`
- **VS Code tasks:** `ast-grep: scan` (lint), `ast-grep: test` (interactive)
- **API docs:** `/api` (Swagger UI from `src/api.rs`)

## Constraints & Conventions

- No HTML caching: Markdown re-parsed per request
- Path security: Canonicalization is best-effort; add strict prefix enforcement if needed
- Template: `meta/html-t.hbs` must exist (auto-created on first run)
- Custom 404: Place `meta/404.html` to override default
- **WHY comments:** Explain rationale, not just what
- **Bilingual docs:** English + 中文 (see `architecture.md`, `src/parser/mod.rs`)
- **Contract sections:** Functions document inputs/outputs/errors/side effects/perf/security
- **Error propagation:** Use `?` with `AppError`; custom `Responder` for HTTP

## Quick References

- Request flow: `docs/request-flow.md`
- Config mapping: `docs/config-templating-map.md`
- Architecture: `architecture.md`
- Key functions: `docs/key-functions.md`
