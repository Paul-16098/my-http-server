<!-- WHY: High-density guide for AI agents; keep ~20–50 lines. Update when routing/templating/middleware changes. -->

## AI Quick Reference (my-http-server)

- Flow: .md → markdown_ppp AST → HTML fragment → Handlebars `meta/html-t.hbs` → response; non-.md → `actix_files::NamedFile` passthrough.
- Key modules: HTTP `src/request.rs`; parsing `src/parser/{markdown.rs,templating.rs,mod.rs}`; config `src/cofg/{config.rs,cli.rs,mod.rs}`; server `src/main.rs`; errors `src/error.rs`.

Configuration & templating

- `Cofg::new()` returns cached config (OnceCell<RwLock<\_>>). No per-request reload API.
- CLI overrides via `cofg::build_config_from_cli` (TLS only enabled when cert+key provided).
- Template engine: global OnceCell<RwLock<Handlebars>>; `templating.hot_reload=true` sets dev_mode (does NOT rebuild each request).
- Context DSL `templating.value`: `name:value` (type inference bool→i64→string) and `name:env:VAR`. Always includes `server-version`; body is injected as HTML fragment.

Routing

- GET / → serve `public/index.html` if present; else build TOC (from `public_path`) and render via `md2html`.
- GET /{filename:.\*} → resolve under `public_path`; if missing prefer `meta/404.html`; if `.md` read + `md2html` with `path:`; if dir → TOC; else stream file.

TOC

- `parser::markdown::get_toc(root, c, title?)` walks with `wax::Glob`, filters by `toc.ext`, ignores `toc.ig`, outputs percent-encoded links.

Middleware order

- Rate limiting → Logger → NormalizePath → Compress → BasicAuth → IP Filter → Routes. Logger uses `%{url}xi` (percent-decoded, trims leading `/`) to target `http-log`.

Dev workflow

- Start: `cargo run` (writes default `cofg.yaml` and `meta/html-t.hbs` on first run and exits—run again).
- Test: `cargo test` (tests in `src/test/*.rs`). Lint: `cargo clippy -- -D warnings`. VS Code tasks: ast-grep: scan/test.

Notes / gotchas

- No HTML render cache; Markdown re-parsed per request.
- Path canonicalization is best-effort; add prefix enforcement if serving untrusted roots.
- Custom 404: create `meta/404.html`.

Quick links: `docs/request-flow.md`, `docs/config-templating-map.md`, `architecture.md`.
