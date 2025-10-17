# Key Functions & Design Rationale

> WHY: Capture the intent behind non-trivial functions to reduce future refactor risk and avoid
> cargo-cult duplication. 中文補充說明輔助溝通。

## Cofg

### `Cofg::new()`

Alias for `get(false)`. Keeps hot path call sites short. Avoids accidental `force_reload` usage that might degrade performance.

### `Cofg::get(force_reload)`

Conditional reload (only if `hot_reload` true). Ensures production config stability while allowing dev refresh.

### `Cofg::force_refresh()`

Bypasses hot_reload guard; test-only / tooling. Not used in runtime code to prevent hidden overhead.

## Templating

### `set_context_value(context, data)`

String mini DSL: `name:value` processed with simple inference.

- Pros: Flexible, no schema explosion
- Cons: Limited types (bool, i64, string) — deliberate for predictability
- Edge Handling: Malformed strings (no ':') ignored silently (safe failure)

### `get_context(cfg)`

Always returns a fresh context (stateless). Ensures no cross-request contamination, simpler mental model vs global mutable context.

### `get_engine(cfg)`

Single global Handlebars engine reused unless `hot_reload`. Rebuild trade-off is deliberate: dev immediacy > raw throughput in that mode.

## Markdown & TOC

### `parser_md(input)`

Encapsulates third-party crate usage & config. Future changes (e.g. enabling extensions) localized here.

### `get_toc(cfg)`

On-demand walk each time index fallback is requested; memoized via bounded LRU keyed by directory mtime + title. Percent-encodes except '/'; ensures link stability across OS path separators.

### `_md2html_all()` / `_make_toc()` (underscored)

Tooling helpers. Not part of server initialization to keep startup constant time.

## Rendering Pipeline

### `md2html(md, cfg, extra_vars)`

Orchestrates: engine retrieval → context creation → AST parse → HTML body injection → template render. Includes an optional HTML output cache keyed by `(abs_path, file_mtime, file_size, template_hbs_mtime, template_ctx_hash)` when `extra_vars` contains a `path:<rel>` that is not a TOC path.

- Accepts owned `md` to avoid clone cost from file read
- Extra variables appended after config-driven ones so request-scoped keys (e.g. path) can override if needed
- Error surfaced as `AppError::Template` for consistent responder logic

## HTTP Layer

### `index` handler

Dual-mode: serve custom `index.html` OR synthesized TOC. Users can introduce bespoke landing page without configuration toggle.

### `main_req` handler

Unified route for all other paths (pattern captures). Single branching logic keeps complexity bounded: existence → markdown? → dynamic render else static file. Custom 404 path tries meta/404.html first to allow styled errors.

## Request Extension Cache (http_ext)

Four derived values precomputed lazily; each micro-optimization prevents repeated computation where logger & handlers might need same values. Memory footprint small (a few strings & paths) per request.

## Error Model

Single `AppError` covers IO / glob / template / markdown / config / other. Downstream complexity shrinks: functions return `AppResult<T>` and rely on `?` propagation. Actix Responder impl ensures consistent 500 behavior for uncaught cases.

## Hot Reload Scope

Only touches config reload & template engine rebuild. Markdown content is always freshly read; no caching ensures the latest file changes are visible regardless of hot_reload.

## Potential Future Evolutions

| Area     | Option                              | Considerations                                          |
| -------- | ----------------------------------- | ------------------------------------------------------- |
| md cache | LRU keyed by (path, mtime)          | Avoid reparse for high-traffic pages; need invalidation |
| partials | Extend DSL for `include:` variables | Keep syntax minimal or move to structured YAML section  |
| search   | Pre-index headings for quick lookup | Likely requires async task & incremental updates        |
| auth     | Middleware gating private docs      | Ensure config reload semantics include auth rules       |
