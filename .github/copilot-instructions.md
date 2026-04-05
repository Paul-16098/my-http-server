# Copilot Instructions for my-http-server

## Project Overview

**my-http-server** is a high-performance Rust HTTP server combining **actix-web**, **markdown-ppp**, **Handlebars templating**, and middleware (auth, rate limiting, IP filtering) for serving static files and dynamically rendering Markdown to HTML.

**Key Goal:** Requests for `.md` files are parsed → converted to HTML → injected into a template (`html-t`) → rendered. All other paths stream as static files.

---

## Architecture Essentials

### Request Pipeline (Critical Flow)

```
HTTP Request
  ↓ [route: / or /{filename:.*}]
  ↓ [cached_public_req_path: decode URI + resolve disk path]
  ├─→ If .md file: md2html(content, config, template_vars) → full HTML
  ├─→ Else if index.html exists: serve raw (user wins)
  ├─→ Else if path is /: generate TOC as markdown → md2html → serve
  └─→ Else: actix_files::NamedFile streaming (404 if missing)
```

### Module Responsibilities (Map Before Editing)

| Module                     | Purpose                                                                              | When to Edit                            |
| -------------------------- | ------------------------------------------------------------------------------------ | --------------------------------------- |
| `src/request.rs`           | Route handlers (`index`, `/{filename}`), path resolution, branching logic            | Adding routes, changing request flow    |
| `src/parser/mod.rs`        | Orchestrates md→HTML→template (`md2html` function)                                   | Changing markdown rendering pipeline    |
| `src/parser/templating.rs` | Handlebars engine lifecycle, context assembly, hot reload logic                      | Template variables, engine lifecycle    |
| `src/parser/markdown.rs`   | TOC generation, markdown parsing utilities                                           | TOC structure, markdown AST transforms  |
| `src/cofg/config.rs`       | Configuration layering (defaults→file→env→CLI), caching via `OnceCell<RwLock<Cofg>>` | Adding config fields, precedence rules  |
| `src/error.rs`             | Unified `AppError` enum + `Responder` impl for HTTP responses                        | Adding error types, status code mapping |
| `src/main.rs`              | Server startup, middleware chain, version info                                       | Adding middleware, initialization       |

---

## Global State & Caching Patterns

### Configuration Caching (Performance Critical)

- **Pattern:** `OnceCell<RwLock<Cofg>>` + lazy initialization
- **Initialization entry point:** `Cofg::init_global(&cli_args, no_xdg)` called once in `main()` **before** handler construction
  - `cli_args`: Parsed CLI arguments (highest precedence)
  - `no_xdg`: Boolean flag; if `true`, skip XDG directory creation (used in tests to avoid I/O)
- **Behavior:**
  - First call to `Cofg::new()` → loads from file/env/CLI per precedence, fills cell
  - Subsequent calls to `Cofg::new()` or `Cofg::get(false)` → cheap clone from cached value (no IO)
  - **Hot reload:** `Cofg::get(true)` forces reload only if `templating.hot_reload=true`
- **Why:** HTTP requests are hot path; avoid per-request config parsing

**Location:** `src/cofg/config.rs::static COFG_ONCE_CELL`

### XDG Paths & File Initialization

- **Pattern:** `Cofg::get_xdg_paths()` returns `Option<XdgPaths>` with 4 canonical locations:
  - `cofg` → Configuration file (`$XDG_CONFIG_HOME/my-http-server/cofg.yaml` or `./cofg.yaml`)
  - `template_hbs` → HTML template (`$XDG_CONFIG_HOME/my-http-server/html-t.hbs` or `./meta/html-t.hbs`)
  - `page_404` → 404 error page (`$XDG_CONFIG_HOME/my-http-server/404.html` or `./meta/404.html`)
  - `emojis` → Emoji cache JSON (only if feature `github_emojis` enabled)
- **Initialization:** `init(&config)` in `main()` creates all XDG directories & default files on first run
- **Why:** Enables XDG-compliant config storage; backward-compatible with local `./meta/` and `./cofg.yaml`

**Location:** `src/cofg/config.rs::get_xdg_paths()` and `src/main.rs::init()`

### Template Engine Caching

- **Pattern:** `OnceCell<RwLock<Handlebars>>`
- **Normal mode:** Built once, reused across all requests (bytecode cached)
- **Hot reload mode:** Rebuilt on every `get_engine()` call to reflect template file changes
- **Why:** Bytecode caching is Handlebars' built-in optimization; hot reload trades performance for dev ergonomics

**Location:** `src/parser/templating.rs::static ENGINE_CACHE`

### Per-Request Caching (http_ext)

Small derived values cached to prevent repeated computation:

- Decoded URI (trim leading `/`)
- Filename PathBuf
- Public absolute path
- Markdown extension check

**Location:** Likely in `src/request.rs` or extension trait pattern

---

## Configuration System (Layered Precedence)

### Initialization Sequence in `main()`

1. **Parse CLI arguments** (via `clap`)
2. **Handle `--root-dir <path>` first** (if provided, changes CWD before config load) — enables deployment in custom directories
3. **Call `Cofg::init_global(&cli_args, false)`** — initiates config layering
4. **Canonicalize `public_path`** — ensures consistent path resolution
5. **Call `init(&config)`** — creates XDG dirs, initializes emoji cache if enabled

### Precedence Order (Lowest → Highest)

1. **Built-in defaults** → `src/cofg/cofg.yaml` (embedded via `include_str!`)
2. **Config file** → `./cofg.yaml`, `--config-path`, or XDG location; skipped if `--no-config` flag set
3. **Environment variables** → `MYHTTP_*` prefix (e.g., `MYHTTP_ADDRS_PORT=3000`)
4. **CLI arguments** → Highest priority (e.g., `--hot-reload true`, `--port 3000`)

### Key Config Sections

- `addrs: {ip, port}` — Listen address
- `tls: {enable, cert, key}` — TLS/HTTPS
- `middleware.*` — Logger, auth, IP filter, compression, path normalization
- `templating: {hot_reload, hbs_path}` — Template engine behavior
- `public_root` — Static file root directory

### Hot Reload Semantics

- **Config:** Reloaded only when `templating.hot_reload=true` AND caller forces `Cofg::get(true)`
- **Template files:** Reloaded when engine is rebuilt (every request in hot reload mode)
- **Template variables:** Always recomputed per request (stateless context)

---

## Context Variable System (Template Data Injection)

### Injection Format: `name:value` Pairs

Three formats supported in `template_data_list`:

1. **Plain:** `name:value` → string literal
2. **Type inference:** Leading digit or 'true'/'false' → auto-detects:
   - `counter:42` → i64
   - `enabled:true` → bool
   - `message:hello` → string
3. **Env substitution:** `name:env:ENV_VAR` → reads `$ENV_VAR`, expands to inferred type

### Context Building Flow (`md2html`)

1. Get engine via `get_engine(config)`
2. Build fresh context via `get_context(config)` → includes built-in keys: `server-version`
3. Apply template_data_list entries (later entries override earlier ones)
4. Parse markdown → AST → HTML fragment
5. Set `context.body = fragment`
6. Render with `html-t` template

**Why:** Keeps context stateless and allows per-request customization without global mutation

**Location:** `src/parser/mod.rs::md2html()` and `src/parser/templating.rs::get_context()`

---

## Error Handling & HTTP Responses

### Pattern: Unified Error Type + Responder

All errors convert to `AppError` enum → implements `actix_web::Responder`:

- **Benefit:** Handlers use `?` operator; no HTTP response coupling in business logic
- **Mapping:** Error variant → HTTP status code (e.g., `Io` → 500, custom logic in `ResponseError` impl)

### Error Variants (src/error.rs)

```rust
pub(crate) enum AppError {
    Io(std::io::Error),                 // 500 Internal Server Error
    TemplateError(handlebars::TemplateError),
    RenderError(handlebars::RenderError),
    MarkdownParseError(String),
    ConfigError(config::ConfigError),
    // ... others
}
```

**When adding errors:**

1. Add variant to `AppError` enum
2. Optionally add custom `#[from]` conversion
3. Implement or derive Display message
4. Update `ResponseError::status_code()` if non-500 mapping needed

### Security: Constant-Time Comparison Functions

- **`constant_time_eq(a: &[u8], b: &[u8]) -> bool`** — Prevents timing attacks on auth secrets by comparing in constant time regardless of mismatch position
- **`ct_eq_str_opt(a: Option<&str>, b: Option<&str>) -> bool`** — Wrapper for optional UTF-8 strings; handles `None` vs `Some` securely

**Usage:** Always use these for comparing credentials in `http_base_authentication` middleware instead of `==`

**Location:** `src/main.rs`

---

## Testing Conventions

### Test Organization (src/test/)

- `cli.rs` — CLI argument parsing
- `config.rs` — Config loading, layering, hot reload
- `error.rs` — Error mappings, status codes
- `integration.rs` — Full HTTP flow (server startup, requests)
- `parser.rs` — Markdown parsing, templating, context assembly
- `request.rs` — Route handlers, path resolution
- `security.rs` — Path traversal, auth, IP filtering
- `main.rs` — Version, initialization

**Guideline:** Tests live in `src/test/{module_name}.rs` matching the corresponding `src/{module_name}.rs`

### Test Setup Pattern

```rust
#[tokio::test]
async fn test_example() {
    // Always call init_test_config() to set up global state (logger, config)
    env_logger::builder().is_test(true).init();
    crate::test::config::init_test_config();

    // Create config with hot_reload disabled for predictable behavior
    let mut config = Cofg::default();
    config.templating.hot_reload = false;
    // ... test logic
}
```

### Running Tests

- **All:** `cargo make test` (wraps `cargo nextest run --all-features`)
- **Coverage:** `cargo make cov` (lcov.info)
- **HTML coverage:** `cargo make html-cov`
- **Specific:** `cargo test --test {name}` or `cargo nextest run {filter}`

### Key Test Guidelines

- Always pass `no_xdg=true` to `Cofg::init_global()` in tests to skip XDG filesystem operations
- Set `templating.hot_reload = false` to avoid cache rebuild flakiness
- Use `init_test_config()` helper to ensure consistent test environment across test suites

---

## Key Patterns & Conventions

### 1. CLI Arguments & Deployment

- **`--root-dir <path>`**: Changes working directory before config load; enables deployment in subdirectories without path munging. Always processed first in `main()`.
- **`--no-config`**: Skips config file entirely; use only defaults + env vars + CLI args (useful for containerized/immutable deploys).
- **`--config-path <path>`**: Override config file location (default: `./cofg.yaml` or XDG location).
- **`--hot-reload <bool>`**: Force enable/disable hot reload (overrides config).
- **Example:** `my-http-server --root-dir /app/srv --no-config --ip 0.0.0.0 --port 8080` for containerized deployment

### 2. Result Type Alias

- Use `type AppResult<T> = Result<T, AppError>` throughout
- Enables `?` operator without wrapping

### 3. Hot Reload Guard

- Always set `hot_reload = false` in tests to avoid flaky behavior from cache rebuilds
- In production, `hot_reload = true` is opt-in via config or CLI

### 4. Path Resolution

- Use `cached_public_req_path()` in handlers to safely decode URI and resolve disk path
- Always validate against `public_root` to prevent path traversal

### 5. Markdown → HTML Pipeline

- Always call `md2html()` function (don't inline engine/context logic)
- Pass template variables as `Vec<String>` with `name:value` syntax
- Template context is immutable per request (no shared mutation)

### 6. Configuration & Initialization

- Call `Cofg::new()` to get cached config (or `Cofg::get(true)` to force reload)
- Call `config.resolve_hbs_path()` to get XDG-aware template path
- Call `config.resolve_404_path()` for 404 page

### 7. Middleware Chain

- Declared in `main.rs::build_server()`
- Order matters: rate limiting → logging → normalize path → compression → auth → IP filter
- Each middleware wraps the previous; error handling is centralized via `AppError` responder

---

## Critical Files (Read Before Large Changes)

1. **`architecture.md`** — Big picture, data flows, hot reload semantics, error propagation
2. **`README.md`** — Tech stack, features, getting started
3. **`src/cofg/config.rs`** — Full config schema + precedence logic
4. **`src/parser/mod.rs`** — md2html contract & side effects
5. **`src/main.rs`** — Server startup, middleware chain, Version info
6. **`src/error.rs`** — Error enum and HTTP status mapping
7. **`Makefile.toml`** — Build tasks (test, coverage, release)

---

## Common Modifications

### Adding a New Configuration Option

1. **Define in `cofg.yaml`** (default schema)
2. **Add field to `Cofg` struct** in `config.rs` with `nest!` or direct field
3. **Parse in config.rs** (usually automatic via serde)
4. **Update `resolve_*` methods** if file paths involved
5. **Add test** in `src/test/config.rs`

### Adding a Route Handler

1. **Define in `src/request.rs`** as async function returning `AppResult<HttpResponse>`
2. **Register in `main.rs::build_server()`** with `.route()` or `.service()`
3. **Use `cached_public_req_path()` or direct path logic**
4. **Return `AppError` on failure** (auto-converts to response via Responder)
5. **Add integration test** in `src/test/integration.rs` or `src/test/request.rs`

### Modifying Markdown Rendering

1. **Update `src/parser/markdown.rs`** for AST transforms
2. **Update `src/parser/templating.rs`** if context variables change
3. **Test in `src/test/parser.rs`**
4. **Check template `meta/html-t.hbs`** matches new context keys

## Enabling Features

- **GitHub Emojis:** Feature `github_emojis` → read `EMOJIS` OnceLock in `parser/mod.rs`
  - On first run, fetches from GitHub API (`https://api.github.com/emojis`).
  - Cache saved to `$XDG_CONFIG_HOME/my-http-server/emojis.json` or `./emojis.json`.
  - Optionally set `GITHUB_TOKEN` env var to avoid API rate limiting.
  - Parses unicode vs. image emoji; separates into `unicode` and `r#else` maps.
- **OpenAPI/Swagger:** Feature `api` → see `src/api/mod.rs`
  - Mounts under `/api` scope; includes Swagger UI at `/api`.
  - Supports read-only file info and listing endpoints.
  - **Run with all:** `cargo test --all-features`

---

## Edge Cases & Gotchas

1. **OnceCell + RwLock Coordination:**
   - Don't hold read lock across await points (deadlock risk)
   - Clone config/engine instead: `let cfg = config.clone(); // cheap`

2. **Hot Reload & Concurrency:**
   - Hot reload rebuilds engine per request (slower but safer for template changes)
   - Set `templating.hot_reload = false` in tests unless testing reload specifically

3. **Path Traversal:**
   - Always call `cached_public_req_path()` which validates against `public_root`
   - Never trust raw request path; always decode and normalize

4. **XDG Config Paths:**
   - Config searches: `$XDG_CONFIG_HOME/my-http-server/cofg.yaml` first, then falls back to `./cofg.yaml`
   - Use `config.resolve_*` methods to respect precedence
   - On first run, `init()` creates all XDG directories + default files automatically

5. **Template File Lazy Registration:**
   - `html-t` template registered on first render only if missing
   - Hot reload triggers re-registration because engine is rebuilt

6. **Emoji Cache Initialization:**
   - Only fetches when cache file is missing (runs once per deployment)
   - Requires network access to GitHub API on first run
   - Can be pre-populated by providing `emojis.json` in config directory

7. **Test XDG Behavior:**
   - Tests use `no_xdg=true` flag to ensure isolation (avoids writing to real XDG paths)
   - If a test needs to verify XDG behavior, handle file I/O manually with `tempfile::TempDir`

---

## Performance Notes (Reference: performance-optimization.instructions.md)

- **Config:** Cached to avoid per-request parsing (use `Cofg::new()`)
- **Template engine:** Bytecode cached by Handlebars (only rebuilt in hot reload mode)
- **Per-request caching:** Decoded URI, resolved paths cached via http_ext pattern
- **Markdown parsing:** Not cached (each render is fresh AST → HTML)
- **Static files:** Streamed via `actix_files::NamedFile` (zero-copy)
- **Middleware:** Ordered for early rejection (IP filter, auth before compression)

---

## Language & Locale Notes

- **Codebase:** Mixed Rust (English comments) + Chinese documentation
- **Comments explaining WHY:** Placed above code sections with `// WHY:` prefix
- **中文部分:** Configuration docs and architecture use Traditional Chinese (TC)
- **When writing:** Match existing style (English for code, Chinese for high-level docs)

---

## Links & References

- **Config schema:** `src/cofg/cofg.yaml`
- **Build tasks:** `Makefile.toml`
- **Architecture & data flow:** `architecture.md`
- **Features & stack:** `README.md`
- **All tests:** `cargo make test`
