<!-- WHY: High-density guide for AI agents; keep ~20–50 lines. Update when routing/templating/middleware changes. -->

## AI Quick Reference (my-http-server)

**Architecture:** Actix-web HTTP server that serves static files and dynamically renders Markdown to HTML via `markdown_ppp` AST + Handlebars templating.

### Request Flow

- `.md` files: read → `markdown_ppp` AST → HTML fragment → inject into `meta/html-t.hbs` (Handlebars) → response
- Other files: stream via `actix_files::NamedFile`
- Missing files: serve `meta/404.html` if present, else plain 404

### Key Modules & Patterns

- **HTTP handling:** `src/request.rs` (routes: `/` index, `/{filename:.*}` catchall), `src/main.rs` (server setup)
- **Parsing pipeline:** `src/parser/mod.rs::md2html` orchestrates markdown→HTML→template; `src/parser/markdown.rs` (TOC generation, parsing); `src/parser/templating.rs` (engine + context)
- **Config:** `src/cofg/config.rs` uses `OnceCell<RwLock<Cofg>>` for global cached config; `Cofg::new()` returns cached value (no per-request reload unless `get(true)` + `hot_reload=true`)
- **Errors:** `src/error.rs` defines `AppError` enum with `Responder` impl; bubble errors with `?` from any depth

### Configuration Gotchas

- **Template context DSL:** `templating.value: ["name:value"]` with type inference (bool → i64 → string) and `name:env:VAR` for env vars
- **Hot reload:** `templating.hot_reload=true` rebuilds Handlebars engine per request (dev only); config reloaded only on explicit `get(true)` call
- **Built-in context vars:** `server-version` (always), `body` (HTML fragment from markdown), `path` (supplied by route handlers)
- **CLI overrides:** `cofg::build_config_from_cli` applies CLI args; TLS enabled only when both cert+key provided

### Routing Behavior

- `GET /` → serve `public/index.html` if exists; else generate TOC from `public_path` via `get_toc` and render with `md2html`
- `GET /{filename:.*}` → resolve under `public_path`; if `.md` → read + `md2html` with `path:` context; if dir → TOC; else static file

### TOC Generation

- `parser::markdown::get_toc(root, c, title?)` walks with `wax::Glob`, filters by `toc.ext`, ignores `toc.ig`, outputs percent-encoded Markdown list

### Middleware Chain (when enabled)

Rate limiting → Logger → NormalizePath → Compress → BasicAuth → IP Filter → Routes

- Logger format uses `%{url}xi` (percent-decoded, leading `/` trimmed) targeting `http-log` module
- Toggle middleware via `cofg.yaml`: `middleware.{normalize_path,compress,logger.enabling,http_base_authentication.enable,ip_filter.enable,rate_limiting.enable}`

### Dev Workflow

- **First run:** `cargo run` writes defaults (`cofg.yaml`, `meta/html-t.hbs`) and exits; run again to start server
- **Tests:** `cargo test` (unit tests in `src/test/*.rs`); integration tests check middleware, auth, IP filter
- **Linting:** `cargo clippy -- -D warnings`
- **VS Code tasks:** `ast-grep: scan` (lint with ast-grep rules), `ast-grep: test` (interactive testing)
- **API docs:** `/api` serves Swagger UI (utoipa-generated from `src/api.rs`)

### Important Constraints

- **No HTML caching:** Markdown re-parsed per request (performance trade-off for simplicity)
- **Path security:** Canonicalization is best-effort; add strict prefix enforcement if serving untrusted roots
- **Template files:** `meta/html-t.hbs` must exist; auto-created from embedded default on first run
- **Custom 404:** Place `meta/404.html` to override default plaintext 404

### Code Conventions

- **WHY comments:** Module and function docs explain rationale, not just what
- **Bilingual docs:** English + 中文 for team accessibility (see `architecture.md`, `src/parser/mod.rs`)
- **Contract sections:** Functions document inputs, outputs, errors, side effects, perf/security notes (see `md2html`)
- **Error propagation:** Use `?` operator with `AppError`; custom `Responder` impl converts to HTTP responses

### Quick References

- Request flow diagrams: `docs/request-flow.md`
- Config→code mapping: `docs/config-templating-map.md`
- Architecture deep dive: `architecture.md`
- Key function list: `docs/key-functions.md`
