# Config ↔ Template & Code Usage Map

> WHY: Provide a single reference mapping `cofg.yaml` fields to their runtime effect / code touch
> points. 中文：快速了解設定欄位如何影響程式行為。

## Top-Level

| Field                        | Type              | Code Reference                                           | Effect                                                                     |
| ---------------------------- | ----------------- | -------------------------------------------------------- | -------------------------------------------------------------------------- |
| `addrs.ip`                   | string            | `main.rs:build_server` via `CofgAddrs`                   | Bind listen IP                                                             |
| `addrs.port`                 | u16               | `main.rs:build_server`                                   | Bind listen port                                                           |
| `middleware.normalize_path`  | bool              | `main.rs:build_server`                                   | Conditionally wraps `NormalizePath(Trim)`                                  |
| `middleware.compress`        | bool              | `main.rs:build_server`                                   | Conditionally wraps `Compress` middleware                                  |
| `middleware.logger.enabling` | bool              | `main.rs:build_server`                                   | Enables `middleware::Logger`                                               |
| `middleware.logger.format`   | string            | `main.rs:build_server`                                   | Passed to `Logger::new` (adds custom url replacement)                      |
| `templating.value`           | list<string>      | `parser/templating.rs:get_context` & `set_context_value` | Provides dynamic template variables (`name:value`)                         |
| `templating.hot_reload`      | bool              | `cofg::get` (reload gate), `templating::get_engine`      | Allows disk reload of config / per-request rebuild of template engine      |
| `toc.path`                   | string (relative) | `markdown.rs:get_toc`, `_make_toc`, `main.rs:index`      | Location (within public) for generated TOC HTML target & base dir for scan |
| `toc.ext`                    | list<string>      | `markdown.rs:get_toc`                                    | File extensions considered for TOC entries                                 |
| `public_path`                | string            | many: `http_ext`, `markdown`, `main`                     | Root directory for content lookup                                          |

## `templating.value` Mini DSL

Format: `name:value`

Resolution order:

1. Split at first ':'
2. If value starts with `env:` then fetch environment variable
3. Try bool parse → try i64 parse → fallback string

Examples:

```yaml
# cofg.yaml snippet
templating:
  value:
    - "feature_x:true" # bool => context.bool(feature_x)=true
    - "build:42" # number => context.number(build)=42
    - "git_hash:env:GIT" # env lookup
    - "title:My Docs" # string
```

## Hot Reload Semantics

| Flag                    | What Triggers Reload                              | Affects                                 |
| ----------------------- | ------------------------------------------------- | --------------------------------------- |
| `templating.hot_reload` | `Cofg::get(true)` calls & every `get_engine` call | Config struct; template engine instance |

中文：若已啟用 hot_reload，程式碼顯式呼叫 `get(true)` 才會重讀設定；模板引擎則每次重新建立，確保檔案修改立即生效。

## Derived Template Variables (Implicit)

| Name             | Source                                                 | Description                               |
| ---------------- | ------------------------------------------------------ | ----------------------------------------- |
| `server-version` | `env!(CARGO_PKG_VERSION)`                              | Crate package version                     |
| `body`           | `md2html` pipeline                                     | Injected rendered HTML fragment           |
| `path`           | Route handlers supply (`path:<req path>` / `path:toc`) | Current logical path for templating logic |

## Touch Points Summary

```text
cofg::get / new --> (read cofg.yaml once; optional reload)
  |         \
  |          -> main.rs (server bind, middleware toggles)
  |          -> templating::get_engine (hot reload decision)
  |          -> templating::get_context (variable list)
  |          -> markdown::get_toc (public_path + toc.*)
  |          -> http_ext (public_path joins)
```

## Validation & Safety Notes

- Missing `cofg.yaml` → program writes embedded default (BUILD_COFG)
- Unknown `templating.value` entry without ':' is ignored (safe no-op)
- `toc.path` must have a parent directory; error otherwise
- `public_path` is created at startup if absent

## Suggestions

| Area       | Improvement                                                                              |
| ---------- | ---------------------------------------------------------------------------------------- |
| Schema     | Optional explicit types (YAML map) for templating values to allow richer numeric support |
| Validation | Add startup validation pass logging warnings for impossible paths / duplicates           |
| Security   | Enforce `public_path` canonical prefix on resolved request paths to mitigate traversal   |
