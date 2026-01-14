<!-- AI Agent Coding Guide: my-http-server (2026-01-13) -->

# AI Coding Agent Instructions

## System Architecture

**Core Purpose:** Rust-based HTTP server (Actix-web 4.11) that serves static files with streaming efficiency and renders Markdown files dynamically via `markdown_ppp` AST + Handlebars templating.

**Request Flow (Markdown):**

```
GET /docs/readme.md → resolve path under public_path → read file
→ md2html(md, config, extra_vars) → parser_md(AST) → render HTML fragment
→ inject into meta/html-t.hbs template + context (path, server-version, configured vars)
→ return full HTML (200 OK)
```

**Request Flow (Static):**

```
GET /assets/logo.png → resolve path → NamedFile::open_async (memory-efficient streaming) → response
```

**Missing Files:** Return custom `meta/404.html` (supports XDG paths) if exists, else plain-text 404.

**Key Caching Strategy:**

- Config: Global `OnceCell<RwLock<Cofg>>` (one-time init + optional hot reload)
- Template Engine: Global `OnceCell<RwLock<Handlebars>>` (rebuilt per request in dev hot-reload mode)
- HTML Output: Optional LRU cache (keyed by `path + mtime + size + template_mtime + ctx_hash`)
- TOC: Optional LRU cache (keyed by `dir_path + dir_mtime + title`)

## Module Responsibilities

| Module                     | Purpose                                            | Key Entry Points                                         | Key Files               |
| -------------------------- | -------------------------------------------------- | -------------------------------------------------------- | ----------------------- |
| `src/main.rs`              | Server composition, middleware chain, logging init | `build_server()`, `init()`, middleware layers            | Actix app factory       |
| `src/request.rs`           | HTTP routes (GET /, GET /{filename:.\*})           | `index()`, `main_req()` handlers                         | Route definitions       |
| `src/parser/mod.rs`        | Orchestrates md→HTML→template pipeline             | `md2html(md, cfg, extra_vars)`                           | Core render entry point |
| `src/parser/markdown.rs`   | Markdown AST parsing, TOC generation, utilities    | `parser_md()`, `get_toc()`, `_md2html_all()`             | AST + TOC logic         |
| `src/parser/templating.rs` | Handlebars engine lifecycle, context assembly      | `get_engine()`, `get_context()`, `set_context_value()`   | Template + context DSL  |
| `src/cofg/config.rs`       | Global config caching + XDG support (OnceCell)     | `Cofg::new()`, `Cofg::get(force)`, `Cofg::init_global()` | Config precedence chain |
| `src/cofg/cli.rs`          | CLI argument parsing (clap derive)                 | `Args` struct & parsing logic                            | Command-line interface  |
| `src/error.rs`             | Unified error type with Responder impl             | `AppError` enum, `AppResult<T>`, `?` propagation         | HTTP status mapping     |

## Rendering Pipeline

**Core Contract** (`md2html` in [src/parser/mod.rs](../src/parser/mod.rs#L43)):

- **Inputs:**
  - `md`: Raw UTF-8 Markdown string; streamed once via `markdown_ppp::parse`.
  - `c: &Cofg`: Read-only config; hot_reload flag determines engine rebuild.
  - `template_data_list: Vec<String>`: Extra context (e.g., `"path:../docs/file.md"`, `"title:env:PAGE_TITLE"`).
- **Output:** Full HTML string via `html-t` template with injected `body` fragment.
- **Type Inference** (in [src/parser/templating.rs](../src/parser/templating.rs)):
  - Format: `name:value` or `name:env:ENV_VAR`.
  - Precedence: `bool` → `i64` → `string`.
  - Malformed entries (no `:`) silently ignored (fail-safe).
- **Errors:** Markdown parse fail / template compile fail / render fail → wrapped as `AppError::RenderError` / `AppError::MarkdownParseError`.
- **Side Effects:** First render lazily registers `html-t` template from disk (via `resolve_hbs_path()`); logs AST at trace level.
- **Performance:** In hot_reload mode (dev), template engine rebuilt per request; HTML caching disabled for safety.

**Context Assembly** ([src/parser/templating.rs](../src/parser/templating.rs)):

- Built-in keys: `server-version` (from `env!(CARGO_PKG_VERSION)`), `body` (rendered HTML).
- Config keys: From `templating.value` DSL.
- Request keys: Caller-supplied (e.g., `path:...`) override config keys.

## Configuration Caching & XDG Support

**Layered Precedence** ([src/cofg/config.rs](../src/cofg/config.rs)):

1. Built-in defaults (embedded `cofg.yaml`)
2. XDG config dir (`~/.config/my-http-server/cofg.yaml` on Linux/macOS; `%LOCALAPPDATA%\my-http-server\config\cofg.yaml` on Windows)
3. Local file (`./cofg.yaml` or `--config-path`)
4. Environment variables (`MYHTTP_*` prefix with `_` separator; e.g., `MYHTTP_ADDRS_IP=0.0.0.0`)
5. CLI arguments (highest priority)

**Global Caching:**

- Stored in `OnceCell<RwLock<GlobalConfig>>` with original CLI args for proper reload.
- `Cofg::init_global(cli_args, no_xdg)` called once at startup to populate.
- Subsequent calls use `Cofg::new()` (cheap clone; ~O(1) per request).
- Hot reload: Only active when `templating.hot_reload=true` in config. Force reload via `Cofg::get(true)` (tests/tools only).

**XDG Path Resolution:**

- `Cofg::resolve_page_404_path()` and `Cofg::resolve_hbs_path()` check local path first, then XDG dir.
- Supports custom 404 pages and templates in XDG location without config file changes.
- WHY: Follow OS conventions; allow site customization without repo modification.

## Error Handling Pattern

**Unified Type** ([src/error.rs](../src/error.rs)):
- Deep functions return `AppResult<T>` (= `Result<T, AppError>`)
- Route handlers use `?` to bubble up; `AppError::Responder` impl ensures consistent HTTP response
- Status mapping: `IO::NotFound` → 404, `IO::PermissionDenied` → 403, most others → 500
- WHY: Decouple error business logic from HTTP; log server-side; respond with generic client message

## Developer Workflow

| Task             | Command                                                       | Notes                                                      |
| ---------------- | ------------------------------------------------------------- | ---------------------------------------------------------- |
| **First run**    | `cargo run` (exits, creates defaults) → run again             | Creates `meta/html-t.hbs`, `cofg.yaml`                     |
| **Development**  | `cargo run`                                                   | Hot reload enabled by default; watch `cofg.yaml` & `meta/` |
| **Test all**     | `cargo make test`                                             | Runs via cargo-make; unit + integration tests              |
| **Coverage**     | `cargo make cov`                                              | Generates LLVM coverage in `target/llvm-cov/html`          |
| **Lint**         | `cargo clippy -- -D warnings`                                 | Must pass to merge; enforces strict checks                 |
| **Format**       | `cargo fmt`                                                   | Checked in CI; ensure idiomatic Rust style                 |
| **Build**        | `cargo build --release`                                       | Optimized binary for production                            |
| **API docs**     | `cargo run --features api` → Open `http://localhost:8080/api`| Swagger UI (requires `api` feature flag)                   |
| **All features** | `cargo run --all-features`                                    | Includes `api` + `github_emojis` features                  |
| **See recipes**  | `cargo make -l`                                               | List all make tasks in `Makefile.toml`                     |

## Feature Flags

- **`api`** (optional): Enables OpenAPI/Swagger UI endpoint at `/api`. Uses `utoipa` to auto-generate docs from code.
- **`github_emojis`** (optional): Fetches GitHub emoji pack from API at startup; requires `GITHUB_TOKEN` env var for higher rate limits.
- **`default`**: Both features enabled by default; disable with `--no-default-features`.

## Coding Conventions

### Documentation & Comments

- **WHY comments only:** Explain rationale, not "what" (code speaks for itself)
- **Contract sections:** Functions document inputs/outputs/errors/side effects/perf/security impacts
- **Bilingual approach:** Core docs in English; comments in `src/` mix English + 中文 for team clarity
- Example: See `src/request.rs` module docs (dual-language flow explanation)

### Error Propagation

- Use `?` with `AppError` for clean call chains
- Never inline HTTP status codes in helpers; let `Responder` impl handle mapping
- Log context (path, config state) before returning errors

### Path Security

- **Current:** Best-effort canonicalization; path joined against configured `public_path`
- **Gap:** No strict canonical prefix check; future hardening needed for untrusted roots
- **Mitigation:** If serving user-provided content roots, add traversal validation in `http_ext`

### Test Organization

- `src/test/mod.rs`: Central module organizing all submodules
- `src/test/config.rs`: Configuration fixture helpers
- `src/test/integration.rs`: Full request→response cycles
- `src/test/parser.rs`: Markdown & templating logic tests
- `src/test/security.rs`: Path traversal & auth tests

### Middleware Chain (configurable via `cofg.yaml`)

```
Rate Limiting → Logger → NormalizePath → Compress → BasicAuth → IP Filter → Routes
```

Each can be toggled; order matters for efficiency (expensive checks after filtering).

## Key Files & References

- **Complete request flow diagram:** [docs/request-flow.md](../docs/request-flow.md)
- **Architecture deep-dive:** [architecture.md](../architecture.md)
- **Config ↔ Code mapping:** [docs/config-templating-map.md](../docs/config-templating-map.md)
- **Function design rationale:** [docs/key-functions.md](../docs/key-functions.md)
- **Performance & caching strategy:** [docs/performance-cache.md](../docs/performance-cache.md)
- **IP filter details:** [docs/ip-filter.md](../docs/ip-filter.md)

## Common Implementation Patterns

### Adding a New Configuration Option

1. Add field to nested struct in [src/cofg/config.rs](../src/cofg/config.rs) (use `nest!` macro for clarity)
2. Add YAML key to [src/cofg/cofg.yaml](../src/cofg/cofg.yaml) (embedded default)
3. Reference via `Cofg::new().field` in hot paths (cached, zero IO)
4. Add integration test in [src/test/config.rs](../src/test/config.rs)
5. Extend middleware chain in `build_server()` if it's a middleware toggle
6. Config can be placed in: XDG config dir (`~/.config/my-http-server/cofg.yaml` or `%APPDATA%\my-http-server\cofg.yaml`), local `./cofg.yaml`, or via CLI args

### Rendering Markdown with Custom Context

```rust
let extra_vars = vec![
    "title:My Page".to_string(),
    "author:env:AUTHOR_NAME".to_string(),
];
let html = md2html(markdown_text, &Cofg::new(), extra_vars)?;
```

### Handling Missing Files & Custom 404

- Route checks `Path::exists()` on resolved path
- If missing, tries `./meta/404.html` (user-customizable)
- Falls back to plain-text "404 Not Found" (see [src/request.rs](../src/request.rs))

### Adding Middleware

1. Toggle in `cofg.yaml::middleware.<name>`
2. Conditional wrap in `build_server()` based on config
3. Order matters: place expensive checks after early rejects (rate limit first)
4. Add integration test exercising the middleware

## Performance & Scalability Notes

- **Markdown rendering cache:** HTML caching by (path, mtime, file_size, template_mtime, context_hash) via LRU (enabled via `cache.enable_html`)
- **TOC caching:** TOC also cached via LRU (enabled via `cache.enable_toc`)
- **Config caching:** Free (RwLock clone is cheap; ~O(1) per request)
- **Engine bytecode:** Handlebars auto-manages; hot reload (dev) rebuilds entire engine per request
- **Streaming:** Static files use `actix_files::NamedFile` (memory-efficient for large assets)
- **Middleware order:** Rate limiting first to reject abuse early; Logger before expensive middleware

## Testing Strategy

- **Unit tests:** Isolate module logic (config, parsing, templating)
  - Example: [src/test/parser.rs](../src/test/parser.rs) — Markdown parse & TOC generation
- **Integration tests:** Full request→response cycles (middleware chain, auth, IP filtering)
  - Example: [src/test/integration.rs](../src/test/integration.rs) — Full HTTP flows
- **Security tests:** Path traversal, authorization edge cases
  - Example: [src/test/security.rs](../src/test/security.rs)
- **Test helpers:** [src/test/config.rs](../src/test/config.rs) provides fixtures (temp dirs, mock requests, default configs)
- **Run:** `cargo make test` (interactive guided exploration) or `cargo make cov` (for coverage report)

## Build & Deployment

- **Debug build:** `cargo build` → faster compile, slower runtime
- **Release build:** `cargo build --release` → slower compile, optimized binary
- **Docker:** [Dockerfile](../Dockerfile) uses multi-stage build for minimal image size
- **TLS:** Configure via `cofg.yaml::tls.{enable,cert,key}` (uses rustls 0.23)
- **Environment:** `RUST_LOG=debug` for verbose logging; `GITHUB_TOKEN` for emoji feature

## Known Limitations & Future Improvements

| Issue                                            | Workaround                           | Priority                  |
| ------------------------------------------------ | ------------------------------------ | ------------------------- |
| No strict canonical prefix check on paths        | Add traversal validation in http_ext | High (security)           |
| Markdown parsing on every request (no AST cache) | Enable `cache.enable_html` flag      | Medium (perf)             |
| Template engine rebuild in hot reload (dev cost) | Acceptable trade-off for DX          | Low (dev-only)            |
| No incremental template streaming                | Not needed for typical doc sizes     | Low (future nice-to-have) |

---

**Last Updated:** 2026-01-13  
**Branch:** dev  
**Version:** 4.1.4
