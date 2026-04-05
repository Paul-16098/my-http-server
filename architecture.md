# Architecture & Data Flow (my-http-server)

> WHY: Provide a precise, implementation-aligned overview so new contributors can reason about
> change impact (config, rendering, HTTP) without diving through all modules. English + 中文混合
> 以便團隊成員快速吸收。

## High-Level Overview

Pipeline (dynamic markdown request):

```text
HTTP Request --> actix-web route --> path resolution (http_ext)
  -> if .md => read file (fs) -> md2html(parser) -> markdown_ppp (AST -> HTML fragment)
  -> templating.get_engine + get_context -> inject body + extras -> Handlebars render
  -> respond HTML
```

Static file path is identical until the extension check branches into `NamedFile::open_async`.

### Module Responsibilities

| Module               | Responsibility                                            | WHY (Rationale)                                              |
| -------------------- | --------------------------------------------------------- | ------------------------------------------------------------ |
| `cofg`               | Load + cache configuration with optional hot reload       | Avoid per-request IO; enable dev tweak cycle                 |
| `parser::templating` | Engine construction & context variable inference          | Keep md2html lean; centralize type inference + env expansion |
| `parser::markdown`   | TOC creation & batch/utility conversion, markdown parsing | Separate tooling from request hot path                       |
| `parser` (mod)       | Orchestrate md→HTML→template pipeline                     | Single entry simplifying handlers                            |
| `http_ext`           | Per-request cached derived values                         | Prevent repeated percent-decode & path joins                 |
| `main`               | Composition of HTTP server & routes                       | Keep side effects (init, server build) contained             |
| `error`              | Unified error type & responder impl                       | Propagate with `?` without HTTP coupling                     |

## Configuration Flow

### Initialization Sequence (main.rs)

1. **Parse CLI arguments** via `clap::Parser` → `cli::Args` struct
2. **Handle `--root-dir <path>` first** → changes working directory before any config loading
   - WHY: Enables deployment in subdirectories without hardcoded paths
   - Example: `my-http-server --root-dir /app/srv` for containerized deployments
3. **Call `Cofg::init_global(&cli_args, no_xdg)`** → initializes global config cache
   - `no_xdg=false` in production → enables XDG directory creation
   - `no_xdg=true` in tests → skips filesystem I/O for isolation
4. **Canonicalize `public_path`** → ensures consistent path resolution across platforms
5. **Call `init(&config)`** → creates XDG directories, initializes emoji cache if feature enabled

### Configuration Precedence (Lowest → Highest)

1. **Built-in defaults** → `src/cofg/cofg.yaml` embedded via `include_str!`
2. **Config file** → searches in order:
   - `--config-path <path>` if provided
   - `$XDG_CONFIG_HOME/my-http-server/cofg.yaml` (Linux/macOS)
   - `./cofg.yaml` (fallback)
   - Skipped entirely if `--no-config` flag set
3. **Environment variables** → `MYHTTP_*` prefix (e.g., `MYHTTP_ADDRS_PORT=8080`)
4. **CLI arguments** → Highest priority (e.g., `--port 3000`, `--hot-reload true`)

### Runtime Behavior

1. First access calls `Cofg::new()` → `OnceCell<RwLock<Cofg>>` filled from layered sources
2. Subsequent `Cofg::new()` calls clone existing value (cheap, small struct) → no disk IO
3. If caller uses `Cofg::get(true)` AND `templating.hot_reload = true`, a fresh struct is reloaded
4. `force_refresh()` (tests) bypasses hot_reload guard

中文：大多數情況以快取設定服務請求；只有熱重載模式才允許明確要求重新載入，以確保生產環境穩定與效能。

### XDG Directory Support

**Pattern:** `Cofg::get_xdg_paths()` returns `Option<XdgPaths>` with 4 canonical locations:

- `cofg` → `$XDG_CONFIG_HOME/my-http-server/cofg.yaml` or `./cofg.yaml`
- `template_hbs` → `$XDG_CONFIG_HOME/my-http-server/html-t.hbs` or `./meta/html-t.hbs`
- `page_404` → `$XDG_CONFIG_HOME/my-http-server/404.html` or `./meta/404.html`
- `emojis` → `$XDG_CONFIG_HOME/my-http-server/emojis.json` or `./emojis.json` (feature `github_emojis`)

**First-run initialization:** `init(&config)` creates all XDG directories + default files automatically

WHY: XDG compliance for Linux/macOS; backward-compatible with local `./meta/` and `./cofg.yaml`

## Template Engine Lifecycle

- Single global `OnceCell<RwLock<Handlebars>>`.
- Normal mode: first build enables bytecode cache; subsequent calls reuse.
- Hot reload mode: every `get_engine` call rebuilds engine (cost accepted for dev ergonomics).
- Context is always fresh (stateless); dynamic variables are re-parsed each request.

## Markdown Rendering Flow

```rust
md2html(md, cfg, extra_vars)
  engine = get_engine(cfg)
  ctx = get_context(cfg)
  for v in extra_vars: set_context_value(ctx, v)
  ast = parser_md(md)
  fragment = markdown_ppp::render_html(ast)
  ctx.body = fragment
  // `html-t` is lazily registered from ./meta/html-t.hbs
  output = engine.render_with_context("html-t", ctx)
```

Errors during compile or render are converted into `AppError::Template` and bubbled up.

## TOC Generation

`get_toc` walks `public_path` for extensions in `toc.ext`, percent-encodes path (except '/') and builds a Markdown list. This is converted to HTML only when needed (index fallback) or via optional tooling `_make_toc`.

## Request Handling Logic

1. `index` route:
   - If `public/index.html` exists → serve raw (user override wins).
   - Else build TOC markdown → `md2html` → serve.
2. `/{filename:.*}` route:
   - Resolve disk path via `cached_public_req_path`.
   - 404 if missing (serve custom meta/404.html if present).
   - If extension `.md` → read file & `md2html` with `path:` variable.
   - Else static file streaming.

### Per-request Caches (http_ext)

- Decoded URI (trim leading '/')
- Filename Path (PathBuf from route param)
- Public absolute path
- Markdown extension boolean

WHY: Small computed values reused across logging, branching, and file reads.

## Hot Reload Semantics

| Feature              | No Hot Reload              | Hot Reload Enabled                         |
| -------------------- | -------------------------- | ------------------------------------------ |
| Config (`Cofg::new`) | Always cached              | Same unless caller forces with `get(true)` |
| Template Engine      | Reused across requests     | Rebuilt each call                          |
| Template Files       | Only read at first compile | Updated every request due to rebuild       |
| Context Vars         | Recomputed per render      | Same                                       |

中文：hot_reload 僅擴散到「需要即時反映變更」的層級（模板與設定），避免不必要的全域重建。

## Error Propagation Strategy

Deep functions return `AppResult<T>`; route handlers pattern-match and translate to HTTP codes (200, 404, 500). `AppError` implements `Responder` which ensures uncaught errors still produce a 500 with logging.

## Concurrency Considerations

- `RwLock` minimizes contention: reads dominate (clone config / read engine); writes occur only on forced reload or hot reload
- Template engine rebuild path obtains a write lock briefly; acceptable due to development-only usage
- **Critical:** Never hold read lock across `.await` points → deadlock risk in async handlers
- **Pattern:** Clone config/engine references before async operations: `let cfg = config.clone();` (cheap Arc-like behavior)

## Middleware Chain

**Order (declared in `main.rs::build_server()`):**

1. **Rate Limiting** (`actix_governor`) → early rejection of excessive requests
2. **Logger** (`middleware::Logger`) → request/response logging
3. **Normalize Path** (`middleware::NormalizePath`) → trailing slash handling
4. **Compress** (`middleware::Compress`) → gzip/brotli response compression
5. **HTTP Basic Auth** (`actix_web_httpauth`) → credential verification with allow/disallow path rules
6. **IP Filter** (`actix_ip_filter`) → whitelist/blacklist rules per path

WHY: Order matters—rate limiting/auth/IP filter reject early to avoid wasting CPU on compression/rendering

中文：中介軟體鏈順序確保昂貴操作（壓縮、渲染）僅應用於通過驗證的請求，提升整體效能與安全性。

## Feature Gates & Conditional Compilation

### `github_emojis` Feature

**Dependencies:** `ureq` for HTTP client

**Initialization:**

1. On first run, checks for emoji cache at XDG or `./emojis.json`
2. If missing, fetches from `https://api.github.com/emojis` (GitHub API)
3. Optionally use `GITHUB_TOKEN` env var to avoid rate limiting
4. Parses response into:
   - `unicode` map: emoji keys → Unicode character strings
   - `r#else` map: emoji keys → GitHub CDN image URLs
5. Saves cache as JSON for subsequent runs

**Location:** `src/parser/mod.rs::EMOJIS` (`OnceLock<Emojis>`)

WHY: One-time fetch reduces GitHub API calls; cache enables offline operation after initial setup

### `api` Feature

**Dependencies:** `utoipa` for OpenAPI schema generation

**Endpoints (mounted at `/api`):**

- `/api` → Swagger UI for interactive API docs
- `/api/openapi.json` → OpenAPI 3.0 schema
- `/api/file/*` → Read-only file info, listing, existence checks
- `/api/meta` → Server metadata (version, build info)
- `/api/license` → License file contents

**Configuration:** `api.enable` in config controls mounting; `api.allow_edit` reserved for future write operations

WHY: Separates API layer from core server; enables programmatic access and testing

## Test Patterns

### XDG Isolation in Tests

**Pattern:** Always pass `no_xdg=true` to `Cofg::init_global()` in tests

```rust
#[tokio::test]
async fn test_example() {
    env_logger::builder().is_test(true).init();
    crate::test::config::init_test_config(); // Sets no_xdg=true
    
    let mut config = Cofg::default();
    config.templating.hot_reload = false; // Avoid flakiness
    // ... test logic
}
```

WHY: Prevents tests from writing to real XDG paths; ensures isolation and repeatability

**XDG verification tests:** Use `tempfile::TempDir` for explicit filesystem testing

## Future Extension Points

| Area          | Suggestion                                               | Notes                                    |
| ------------- | -------------------------------------------------------- | ---------------------------------------- |
| Watcher       | Optional file watcher triggering `Cofg::force_refresh()` | Add feature flag to avoid runtime cost   |
| Pre-render    | CLI command to batch md→html for static hosting          | Reuse `_md2html_all()`                   |
| Caching layer | (done) HTML render LRU keyed by path/mtime/size/template | Avoid repeated parsing for popular pages |
| Streaming     | Output chunked HTML while rendering large docs           | Requires incremental printer support     |
| Metrics       | Add request/render timing histogram                      | Expose via /metrics (Prometheus)         |
| Multi-tenant  | Support multiple `public_path` roots with path routing   | Requires config schema extension         |

## Security Notes

### Path Traversal Protection

- Paths are joined against configured `public_path` via `cached_public_req_path()`
- Request URIs are percent-decoded and normalized
- All resolved paths are validated against canonicalized `public_root`
- WHY: Prevents `../` attacks and ensures files are served only from allowed directory

### Constant-Time Credential Comparison

**Functions (src/main.rs):**

- `constant_time_eq(a: &[u8], b: &[u8]) -> bool` — byte-level constant-time comparison
- `ct_eq_str_opt(a: Option<&str>, b: Option<&str>) -> bool` — wrapper for optional strings

**Usage:** Always use these for comparing credentials in `http_base_authentication` middleware instead of `==`

WHY: Prevents timing attacks where attacker measures response time to infer password characters

**Implementation:** Compares full length of both inputs regardless of mismatch position; XORs mismatched bytes into accumulator to ensure constant-time execution

### Template Variable Trust

- Template variables from config are trusted (admin-controlled)
- If extended to user input, sanitize before `set_context_value` to prevent template injection
- Context keys like `server-version` are hardcoded; user-provided keys override config values (last-write-wins)

## CLI Arguments for Deployment

### Critical Flags

- `--root-dir <path>` → Changes working directory **before** config load; enables deployment in custom locations
- `--no-config` → Skips config file entirely; uses only defaults + env vars + CLI args (immutable deployments)
- `--config-path <path>` → Override config file location (default: `./cofg.yaml` or XDG)
- `--hot-reload <bool>` → Force enable/disable hot reload (overrides config)

### Deployment Examples

**Containerized (Docker):**

```bash
my-http-server --root-dir /app/srv --no-config --ip 0.0.0.0 --port 8080
```

**Systemd Service:**

```bash
my-http-server --config-path /etc/my-http-server/cofg.yaml --public-path /var/www/docs
```

**Development:**

```bash
my-http-server --hot-reload true --port 3000
```

WHY: `--root-dir` processed first allows config/templates/static files to be relative to deployment location without hardcoded paths

## Summary

System prefers simplicity & per-request clarity over aggressive caching except where global reuse is nearly free (config, engine). Hot reload mode purposefully narrows extra cost to development scenarios.

**Key architectural decisions:**

- **Layered config precedence** → Flexible deployment without code changes
- **XDG compliance** → Linux/macOS best practices + backward compatibility
- **Constant-time auth** → Security-first credential comparison
- **Middleware ordering** → Performance through early rejection
- **Feature gates** → Optional dependencies reduce binary size
- **Test isolation** → `no_xdg` flag prevents filesystem pollution

中文：架構優先考慮可部署性與安全性，同時保持程式碼簡潔與測試隔離。設定層級與 XDG 支援使系統適應多種部署場景。
