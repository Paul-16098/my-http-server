# Architecture & Data Flow (my-http-server)

> WHY: Provide a precise, implementation-aligned overview so new contributors can reason about
> change impact (config, rendering, HTTP) without diving through all modules. English + 中文混合
> 以便團隊成員快速吸收。

## High-Level Overview

Pipeline (dynamic markdown request):

```
HTTP Request --> actix-web route --> path resolution (http_ext)
  -> if .md => read file (fs) -> md2html(parser) -> markdown_ppp (AST -> HTML fragment)
      -> templating.get_engine + get_context -> inject body + extras -> mystical_runic render
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

1. First access calls `Cofg::new()` → `OnceCell<RwLock<Cofg>>` filled from `cofg.yaml` (or embedded default).
2. Subsequent `Cofg::new()` calls clone existing value (cheap, small struct) → no disk IO.
3. If caller uses `Cofg::get(true)` AND `templating.hot_reload = true`, a fresh struct is reloaded.
4. `force_refresh()` (tests) bypasses hot_reload guard.

中文：大多數情況以快取設定服務請求；只有熱重載模式才允許明確要求重新載入，以確保生產環境穩定與效能。

## Template Engine Lifecycle

- Single global `OnceCell<RwLock<TemplateEngine>>`.
- Normal mode: first build enables bytecode cache; subsequent calls reuse.
- Hot reload mode: every `get_engine` call rebuilds engine (cost accepted for dev ergonomics).
- Context is always fresh (stateless); dynamic variables are re-parsed each request.

## Markdown Rendering Flow

```
md2html(md, cfg, extra_vars)
  engine = get_engine(cfg)
  ctx = get_context(cfg)
  for v in extra_vars: set_context_value(ctx, v)
  ast = parser_md(md)
  fragment = markdown_ppp::render_html(ast)
  ctx.body = fragment
  template = engine.compile_to_bytecode("html-t.templating") // cached in engine layer
  output = engine.render(template, ctx)
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

- `RwLock` minimizes contention: reads dominate (clone config / read engine); writes occur only on forced reload or hot reload.
- Template engine rebuild path obtains a write lock briefly; acceptable due to development-only usage.

## Future Extension Points

| Area          | Suggestion                                               | Notes                                    |
| ------------- | -------------------------------------------------------- | ---------------------------------------- |
| Watcher       | Optional file watcher triggering `Cofg::force_refresh()` | Add feature flag to avoid runtime cost   |
| Pre-render    | CLI command to batch md→html for static hosting          | Reuse `_md2html_all()`                   |
| Caching layer | Add rendered HTML cache keyed by file mtime              | Avoid repeated parsing for popular pages |
| Streaming     | Output chunked HTML while rendering large docs           | Requires incremental printer support     |
| Metrics       | Add request/render timing histogram                      | Expose via /metrics (Prometheus)         |

## Security Notes

- Paths are joined against configured `public_path`; no explicit sanitization of `..`—consider enforcing canonicalization & prefix check before access.
- Template variables from config are trusted; if extended to user input, sanitize before `set_context_value`.

## Summary

System prefers simplicity & per-request clarity over aggressive caching except where global reuse is nearly free (config, engine). Hot reload mode purposefully narrows extra cost to development scenarios.
