# Developer Guide (深入開發指南)

> Goal: Accelerate onboarding with practical tasks & mental models. 目標：快速進入狀況。

## 1. Quick Start

```bash
cargo run
# visit http://127.0.0.1:8080 (or configured ip/port)
```

If `cofg.yaml` missing it is auto-created from embedded default.

## 2. Key Mental Models

| Concept         | Think Of It As                      | Notes                                      |
| --------------- | ----------------------------------- | ------------------------------------------ |
| Config (`Cofg`) | Immutable snapshot (cheap clone)    | Reload only under hot reload explicit call |
| Template Engine | Heavy object w/ bytecode cache      | Rebuilt each request only in hot_reload    |
| md2html         | Pure transformation + template wrap | No global state modification               |
| http_ext caches | Per-request memoization             | Avoid redoing string/path work             |

## 3. Common Tasks

### Add New Template Variable

1. Append string to `templating.value` in `cofg.yaml` (e.g. `- "docs_mode:true"`)
2. (Optional) Use `env:` prefix for environment injection
3. Reference in `meta/html-t.templating` via `{{ docs_mode }}`
4. Enable `templating.hot_reload: true` during development to avoid restart

### Expose Config Field to Templates

If it's a dynamic toggle better as `templating.value`. Only add new struct fields when the value has strong type semantics or needed by Rust code.

### Add Middleware

`build_server` in `main.rs` composes middleware under conditional wrappers. Pattern:

```rust
.wrap(middleware::Condition::new(cfg_flag, middleware::SomeMiddleware::new()))
```

### Batch Convert Markdown (static export)

Call tooling helpers (manually or via future CLI command):

- `_md2html_all()`
- `_make_toc()`

### Change 404 Page

Edit `meta/404.html`. No restart needed if only static file; templates not involved.

## 4. Testing Strategy

| Layer          | Test Type                       | Example                            |
| -------------- | ------------------------------- | ---------------------------------- |
| Config load    | Unit                            | Default values, hot reload gating  |
| Templating     | Unit                            | `set_context_value` parsing matrix |
| Markdown parse | Unit                            | Edge syntax constructs             |
| md2html        | Integration                     | Template error propagation         |
| HTTP           | Integration (actix test server) | Markdown vs static path branching  |

Use `cargo nextest run` for faster & nicer output.

## 5. Style & Conventions

| Area              | Convention                                                                               |
| ----------------- | ---------------------------------------------------------------------------------------- |
| Error propagation | Prefer `?` returning `AppResult<T>`                                                      |
| Logging           | `debug!` for flow; `warn!` for recoverable issues; `error!` for template/render failures |
| File naming       | Keep modules short (`cofg`, `parser`, `http_ext`)                                        |
| Internal tooling  | Prefix with `_` to mark non-route usage                                                  |

## 6. Extending Functionality

### Add CLI Flags

Edit `src/cofg/cli.rs` (not shown here) then merge into config via `build_config_from_cli`.

### Add New Output Format

Keep `md2html` pure; create new function (e.g. `md2pdf`) that parallels pipeline & reuses `parser_md`.

### Add Live Watcher

Potential crate: `notify`. Flow:

1. Spawn background task on startup when feature enabled
2. Watch `cofg.yaml` & `meta/` & `public/`
3. On change: `Cofg::force_refresh()` or invalidate HTML cache (future)

### Implement HTML Render Cache

Pseudo-code:

```rust
struct RenderCacheKey { path: PathBuf, mtime: SystemTime }
// HashMap<RenderCacheKey, Arc<String>>
```

Look up before `read_to_string`; if mtime changed re-render.

## 7. Performance Checklist

| Check                 | Action                              |
| --------------------- | ----------------------------------- |
| High CPU parse        | Add render cache                    |
| Slow TOC              | Precompute at startup or memoize    |
| Template thrash (dev) | Confirm hot_reload only when needed |

## 8. Security Considerations

| Risk                                  | Mitigation Idea                     |
| ------------------------------------- | ----------------------------------- |
| Path traversal (`..`)                 | Canonicalize & enforce prefix guard |
| Large untrusted files                 | Add size cap before reading         |
| Template injection (future user vars) | Escape or whitelist variable names  |

## 9. Troubleshooting

| Symptom                  | Likely Cause             | Fix                                  |
| ------------------------ | ------------------------ | ------------------------------------ |
| Template changes ignored | hot_reload false         | Set `templating.hot_reload: true`    |
| Config not updating      | Using `Cofg::new()` only | Call `Cofg::get(true)` in admin path |
| 404 Always               | Wrong `public_path`      | Check or recreate directory          |
| Garbled UTF-8            | Source file encoding     | Ensure UTF-8 without BOM             |

## 10. Future CLI Ideas

| Command       | Purpose                          |
| ------------- | -------------------------------- |
| `--export`    | Batch produce static HTML site   |
| `--list-vars` | Print resolved templating values |
| `--validate`  | Run config + path validations    |

## 11. Contribution Flow

1. Fork / branch from `dev`
2. Add or update tests under `src/test/`
3. Run `cargo fmt` & `cargo clippy -- -D warnings` (add if not in CI)
4. Run `cargo nextest run`
5. Open PR describing rationale (reference doc section if relevant)

## 12. Glossary

| Term           | Meaning                                                                           |
| -------------- | --------------------------------------------------------------------------------- |
| Hot Reload     | Rebuild template engine & allow config re-read (on explicit demand) every request |
| Bytecode Cache | Precompiled template bytecode stored in memory for faster render                  |
| TOC            | Table of Contents (generated markdown listing)                                    |

## 13. Release Notes

### 3.0.2 (2025-09-29)

Focus: performance (per-request caching), documentation consolidation, and release automation hardening.

Highlights:

- Perf: Introduced per-request helper caching in `http_ext` to avoid repeated path/extension decoding work.
- Docs: Added/updated architecture, request flow, key functions, performance cache rationale; unified developer guide.
- CI/Release: Improved release workflow, added broader GPG detached signatures matching pattern, refined build metadata.
- Automation: Renovate configuration added for ongoing dependency maintenance.
- Dependencies: Bumped core libraries (templating, parsing, config, CLI, error handling) to latest compatible versions.

Internal Notes:

- Build script embeds `VERSION` = crate version + profile + short commit hash for traceability.
- No breaking API changes; minor version bump stays within 3.x contract.
- Future ideas: render cache & static export CLI flag (`--export`).

Upgrade Guidance:

- No action required; rebuild to benefit from caching & updated deps.
- Optionally review new docs sections for mental model refresh.

