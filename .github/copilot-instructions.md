<!-- WHY: High-density guide for AI agents; update when routing/templating/caching/middleware changes. Keep ~20-50 lines. -->

## AI Development Quick Reference (my-http-server)

### Architecture & Hot Paths

**Pipeline**: `.md` files → markdown-ppp AST → HTML fragment → Handlebars template (`meta/html-t.hbs`) → response
**Static files**: Direct passthrough via `actix_files::NamedFile`

**Core modules**:
- HTTP: `src/request.rs` (routes), `src/http_ext.rs` (per-request caches), `src/main.rs` (server composition)
- Parsing: `src/parser/{mod.rs,markdown.rs,templating.rs}`
- Config: `src/cofg/{config.rs,cli.rs,mod.rs}`

### Critical Patterns

**Configuration caching**: Always use `Cofg::new()` in hot paths—returns cached `OnceCell<RwLock<_>>` clone. Force reload only with `Cofg::get(true)` AND `templating.hot_reload=true`.

**Template engine lifecycle**: Global `OnceCell<RwLock<Handlebars>>` with bytecode cache. Rebuilds per-request ONLY when `hot_reload=true`.

**Per-request caches** (via `HttpRequestCachedExt` trait in `http_ext.rs`):
```rust
req.cached_filename_path()      // PathBuf from route param
req.cached_public_req_path(c)   // Joined with public_path
req.cached_is_markdown(c)       // Extension check (.md)
```

**Template context DSL** (`templating.value` in config):
- `name:value` → infers bool/i64/string
- `name:env:VAR` → reads env var then infers type
- Example: `docs_mode:true` → accessible as `{{docs_mode}}` in templates

**Cross-request caches** (LRU):
- HTML: Keyed by `(abs_path, file_mtime, file_size, template_mtime, ctx_hash)`, controlled by `cache.enable_html`
- TOC: Keyed by `(dir_abs, dir_mtime, title)`, controlled by `cache.enable_toc`

### Routing Logic

`GET /`:
1. If `public/index.html` exists → serve static
2. Else → `get_toc(public_path)` → `md2html` with template

`GET /{filename:.*}`:
1. Resolve to `public_path` + decoded filename
2. If missing → try `meta/404.html` → else plain 404
3. If `.md` → `read_to_string` → `md2html(path:<rel>)`
4. If directory → `get_toc(dir)` → `md2html(path:toc:<dir>)`
5. Else → `NamedFile::open_async`

### Middleware Order & Config

Execution order (when enabled): Rate limiting → Logger → NormalizePath → Compress → BasicAuth → IP Filter → Routes

**Key middleware configs**:
- `middleware.rate_limiting.{seconds_per_request,burst_size}` (actix-governor)
- `middleware.http_base_authentication` uses constant-time password comparison; `allow` rules override `disallow`
- `middleware.ip_filter.{allow,block}` accepts glob patterns; runs AFTER BasicAuth
- Logger uses `%{url}xi` (percent-decoded, leading `/` removed, target=`http-log`)

### Development Workflows

**Start**: `cargo run` (auto-creates `cofg.yaml` if missing; generates default `meta/html-t.hbs` on first run and exits)

**Test**: `cargo test` (tests in `src/test/*.rs`); VS Code tasks: "ast-grep: test" (interactive), "ast-grep: scan" (quick lint)

**Lint**: `cargo clippy -- -D warnings` (CI standard)

**Build release**: `cargo build --release`

**Docker**: Mount volumes for `public/` and optionally `meta/`; use `--ip 0.0.0.0` in containers

### Common Gotchas

- **Template changes ignored**: Enable `templating.hot_reload=true` OR restart server
- **Stale HTML cache**: Verify `cache.enable_html=true` and check file mtime/size in cache key
- **404 shows plain text**: Ensure `meta/404.html` exists
- **Path traversal risk**: No canonical-prefix enforcement yet—validate untrusted paths manually

### Quick Navigation

- Request flow: `docs/request-flow.md`
- Cache strategy: `docs/performance-cache.md`
- Config mapping: `docs/config-templating-map.md`
- Architecture: `architecture.md`
- Full guide: `docs/developer-guide.md`
