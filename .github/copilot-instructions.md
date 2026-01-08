<!-- AI Agent Coding Guide: my-http-server (2026.01) -->

# AI Coding Agent Instructions

## System Architecture

**Core Purpose:** Actix-web HTTP server that serves static files and renders Markdown files dynamically via `markdown_ppp` AST + Handlebars templating.

**Request Flow (Markdown):**

```
GET /docs/readme.md → http_ext per-request cache → resolve path 
→ read file → parse (markdown_ppp AST) → render to HTML fragment 
→ inject into meta/html-t.hbs template → add context (path, server-version, configured vars) 
→ return full HTML
```

**Request Flow (Static):**

```
GET /assets/logo.png → http_ext per-request cache → resolve path 
→ actix_files::NamedFile (streaming) → response
```

**Missing Files:** Return custom `meta/404.html` if exists, else plain-text 404.

## Module Responsibilities

| Module                     | Purpose                                            | Key Entry Points                                       |
| -------------------------- | -------------------------------------------------- | ------------------------------------------------------ |
| `src/main.rs`              | Server composition, middleware chain, logging init | `build_server()`, `init()`, middleware layers           |
| `src/request.rs`           | HTTP routes (GET /, GET /{filename:.*})            | `index()`, `main_req()` handlers                       |
| `src/parser/mod.rs`        | Orchestrates md→HTML→template pipeline             | `md2html(md, cfg, extra_vars)`                         |
| `src/parser/markdown.rs`   | Markdown AST parsing, TOC generation, utilities    | `parser_md()`, `get_toc()`, `_md2html_all()`           |
| `src/parser/templating.rs` | Handlebars engine lifecycle, context assembly      | `get_engine()`, `get_context()`, `set_context_value()` |
| `src/cofg/config.rs`       | Global config caching (OnceCell<RwLock<Cofg>>)     | `Cofg::new()`, `Cofg::get(force)`, hot reload guards   |
| `src/cofg/cli.rs`          | CLI argument parsing                               | `CliArgs` struct & parsing logic                       |
| `src/error.rs`             | Unified error type with Responder impl             | `AppError` enum, `AppResult<T>`, `?` propagation       |
| `src/http_ext.rs`          | Per-request cached path/URI derivations            | `cached_*` functions (DecodedUri, FilenamePath, etc.)   |

## Critical Data Structures & Patterns

### Configuration Caching (`Cofg::new()` & `Cofg::get()`)

- Stored in global `OnceCell<RwLock<Cofg>>`
- **Normal mode:** Returns cached clone (zero IO cost)
- **Hot reload mode** (`templating.hot_reload=true`, dev only): Caller can force reload via `get(true)`
- **Why:** Keep hot paths fast (99% of requests use cached config); dev ergonomics without production cost

### Template Engine Lifecycle (`get_engine()`)

- Single global `OnceCell<RwLock<Handlebars>>`
- **Normal mode:** First build caches bytecode; reused thereafter
- **Hot reload mode:** Entire engine rebuilt per request (intentional dev cost)
- **Template registration:** `html-t` lazily registered from `./meta/html-t.hbs` on first render

### Context DSL (`templating.value` in config)

```yaml
templating:
  value:
    - "name:value" # string
    - "count:42" # auto-inferred as i64
    - "enabled:true" # auto-inferred as bool
    - "apikey:env:API_KEY" # pulled from environment
```

Type inference order: `bool` → `i64` → `string`. Malformed entries silently ignored.

### Per-Request HTTP Extensions (`http_ext` module)

Caches derived values to prevent repeated computation:

| Cached Value       | Purpose                          | Used By                |
| ------------------ | -------------------------------- | ---------------------- |
| `DecodedUri`       | Percent-decoded, trailing-slash  | Logger, branching      |
| `FilenamePath`     | PathBuf from route parameter     | Path resolution        |
| `PublicReqPath`    | Resolved absolute path under pub | File existence checks  |
| `IsMarkdown`       | File extension check (`.md`)     | Route branching        |

**Why:** Small structs + cheap clones allow handlers, logger, and error paths to avoid recomputation.

## Error Handling Pattern

```rust
// Deep functions return AppResult<T> (= Result<T, AppError>)
fn helper() -> AppResult<String> { ... }

// Route handlers use ? to bubble up
#[post("/")]
async fn my_route() -> AppResult<impl Responder> {
    let data = helper()?;  // Error auto-converts to HTTP 500 + logging
    Ok(HttpResponse::Ok().json(data))
}

// AppError::Responder impl ensures uncaught errors still produce proper HTTP response
```

## Developer Workflow

| Task            | Command                                                  | Notes                                                    |
| --------------- | -------------------------------------------------------- | -------------------------------------------------------- |
| **First run**   | `cargo run` (exits, creates defaults) → run again        | Creates `meta/html-t.hbs`, `cofg.yaml`                   |
| **Development** | `cargo run`                                              | Hot reload enabled by default; watch `cofg.yaml` & `meta/`|
| **Test all**    | `cargo nextest run` or `cargo make test`                 | Tests in `src/test/*.rs` (unit + integration)             |
| **Lint**        | `cargo clippy -- -D warnings`                            | Must pass to merge; enforces strict checks                |
| **Format**      | `cargo fmt`                                              | Checked in CI; ensure idiomatic Rust style               |
| **Build**       | `cargo build --release`                                  | Optimized binary for production                          |
| **API docs**    | `cargo run --features api` → Open `http://localhost/api` | Swagger UI (requires `api` feature flag)                 |
| **All features**| `cargo run --all-features`                               | Includes `api` + `github_emojis` features                |

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
- **Run:** `cargo nextest run` (recommended) or `cargo make test` (interactive guided exploration)

## Build & Deployment

- **Debug build:** `cargo build` → faster compile, slower runtime
- **Release build:** `cargo build --release` → slower compile, optimized binary
- **Docker:** [Dockerfile](../Dockerfile) uses multi-stage build for minimal image size
- **TLS:** Configure via `cofg.yaml::tls.{enable,cert,key}` (uses rustls 0.23)
- **Environment:** `RUST_LOG=debug` for verbose logging; `GITHUB_TOKEN` for emoji feature

## Known Limitations & Future Improvements

| Issue                                     | Workaround                           | Priority          |
| ----------------------------------------- | ------------------------------------ | ------------------|
| No strict canonical prefix check on paths | Add traversal validation in http_ext | High (security)   |
| Markdown parsing on every request (no AST cache) | Enable `cache.enable_html` flag | Medium (perf)     |
| Template engine rebuild in hot reload (dev cost) | Acceptable trade-off for DX | Low (dev-only)     |
| No incremental template streaming         | Not needed for typical doc sizes     | Low (future nice-to-have) |

---

**Last Updated:** 2026-01-08  
**Branch:** dev  
**Version:** 4.1.0
