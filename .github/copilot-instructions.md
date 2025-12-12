<!-- AI Agent Coding Guide: my-http-server (2025.12) -->

# AI Coding Agent Instructions

## System Architecture

**Core Purpose:** Actix-web HTTP server that serves static files and renders Markdown files dynamically via `markdown_ppp` AST + Handlebars templating.

**Request Flow (Markdown):**

```
GET /docs/readme.md → resolve path → read file → parse (markdown_ppp AST)
→ render to HTML fragment → inject into meta/html-t.hbs template
→ add context (path, server-version, configured vars) → return full HTML
```

**Request Flow (Static):**

```
GET /assets/logo.png → resolve path → actix_files::NamedFile (streaming) → response
```

**Missing Files:** Return custom `meta/404.html` if exists, else plain-text 404.

## Module Responsibilities

| Module                     | Purpose                                            | Key Entry Points                                       |
| -------------------------- | -------------------------------------------------- | ------------------------------------------------------ |
| `src/main.rs`              | Server composition, middleware chain, logging init | `create_server()`, middleware layers                   |
| `src/request.rs`           | HTTP routes (GET /, GET /{filename:.\*})           | `index()`, `fallback()` handlers                       |
| `src/parser/mod.rs`        | Orchestrates md→HTML→template pipeline             | `md2html(md, cfg, extra_vars)`                         |
| `src/parser/markdown.rs`   | Markdown AST parsing, TOC generation               | `parser_md()`, `get_toc()`                             |
| `src/parser/templating.rs` | Handlebars engine lifecycle, context assembly      | `get_engine()`, `get_context()`, `set_context_value()` |
| `src/cofg/config.rs`       | Global config caching (OnceCell<RwLock<\_>>)       | `Cofg::new()`, hot reload guards                       |
| `src/error.rs`             | Unified error type with Responder impl             | `AppError` enum, `?` propagation                       |

## Critical Data Structures & Patterns

### Configuration Caching (`Cofg::new()`)

- Stored in global `OnceCell<RwLock<Cofg>>`
- **Normal mode:** Returns cached clone (zero IO cost)
- **Hot reload mode** (`templating.hot_reload=true`, dev only): Caller can force reload via `get(true)`
- **Why:** Keep hot paths fast; enable dev ergonomics without production cost

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

### Per-Request Caching

`http_ext` module caches small derived values (percent-decoded URI, resolved PathBuf, canonical path) to avoid recomputation across logging, branching, and file I/O.

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

| Task            | Command                                           | Notes                                                              |
| --------------- | ------------------------------------------------- | ------------------------------------------------------------------ |
| **First run**   | `cargo run` (exits, creates defaults) → run again | Creates `meta/html-t.hbs`, `cofg.yaml`                             |
| **Development** | `cargo run`                                       | Hot reload enabled by default; watch `cofg.yaml` & `meta/` changes |
| **Test**        | `cargo test` or `cargo make test`                 | Tests in `src/test/*.rs` (unit + integration)                      |
| **Lint**        | `cargo clippy -- -D warnings`                     | Must pass to merge                                                 |
| **Format**      | `cargo fmt`                                       | Checked in CI                                                      |
| **API docs**    | Open `/api` endpoint                              | Swagger UI auto-generated from `src/api.rs`                        |

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
- **Mitigation:** If serving user-provided content roots, add traversal validation

### Middleware Chain (configurable via `cofg.yaml`)

```
Rate Limiting → Logger → NormalizePath → Compress → BasicAuth → IP Filter → Routes
```

Each can be toggled; order matters for efficiency (expensive checks after filtering).

## Key Files & References

- **Request flow:** [docs/request-flow.md](../docs/request-flow.md)
- **Architecture deep-dive:** [architecture.md](../architecture.md)
- **Config mapping:** [docs/config-templating-map.md](../docs/config-templating-map.md)
- **Key functions:** [docs/key-functions.md](../docs/key-functions.md)
- **Markdown parsing:** [src/parser/markdown.rs](../src/parser/markdown.rs)
- **Templating DSL:** [src/parser/templating.rs](../src/parser/templating.rs)

## Common Implementation Patterns

### Adding a New Configuration Option

1. Add field to nested struct in `src/cofg/config.rs` (use `nest!` macro for clarity)
2. Add YAML key to `src/cofg/cofg.yaml` (embedded default)
3. Reference via `Cofg::new().field` in hot paths (cached, zero IO)
4. Extend middleware chain if it's a middleware toggle

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
- Falls back to plain-text "404 Not Found"

## Performance & Scalability Notes

- **Markdown caching:** Removed (every request re-parses); future optimization: cache by (path, mtime, size, template_version)
- **Config caching:** Free (RwLock clone is cheap)
- **Engine bytecode:** Handlebars auto-manages; hot reload disables for dev
- **Streaming:** Static files use `actix_files::NamedFile` (efficient for large assets)
- **Middleware order:** Rate limiting first to reject abuse early; Logger before expensive middleware

## Testing Strategy

- **Unit tests:** Isolate module logic (config, parsing, templating)
- **Integration tests:** Full request→response cycles (middleware chain, auth, IP filtering)
- **Test helpers:** `src/test/` modules provide fixtures (temp dirs, mock requests, default configs)
- **Run interactive:** `cargo make test` for guided test exploration
