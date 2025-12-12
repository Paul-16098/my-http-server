# Config ↔ Template & Code Usage Map

> WHY: Provide a single reference mapping `cofg.yaml` fields to their runtime effect / code touch
> points. 中文：快速了解設定欄位如何影響程式行為。

# Config ↔ Template & Code Usage Map

> WHY: 提供 cofg.yaml 配置项与代码行为的唯一映射参考，便于开发、调试和扩展。
> 中文：快速了解配置字段如何影响程序行为，支持双语协作。

> 最后更新时间：2025-12-13
> 适用版本：dev 分支
> 参见：`.github/copilot-instructions.md`（AI 编码代理完整指引）

## Top-Level

| Field                        | Type              | Code Reference                                           | Effect                                                                     |
| ---------------------------- | ----------------- | -------------------------------------------------------- | -------------------------------------------------------------------------- |
| `addrs.ip`                   | string            | `main.rs:build_server` via `CofgAddrs`                   | Bind listen IP                                                             |
| `addrs.port`                 | u16               | `main.rs:build_server`                                   | Bind listen port                                                           |
| `tls.enable`                 | bool              | `main.rs:build_server`                                   | Enable TLS/HTTPS; when true, uses `bind_rustls_0_23` instead of `bind`     |
| `tls.cert`                   | string            | `main.rs:load_tls_config`                                | Path to TLS certificate file (PEM format)                                  |
| `tls.key`                    | string            | `main.rs:load_tls_config`                                | Path to TLS private key file (PEM format)                                  |
| `middleware.normalize_path`  | bool              | `main.rs:build_server`                                   | Conditionally wraps `NormalizePath(Trim)`                                  |
| `middleware.compress`        | bool              | `main.rs:build_server`                                   | Conditionally wraps `Compress` middleware                                  |
| `middleware.logger.enabling` | bool              | `main.rs:build_server`                                   | Enables `middleware::Logger`                                               |
| `middleware.logger.format`   | string            | `main.rs:build_server`                                   | Passed to `Logger::new` (adds custom url replacement)                      |
| `templating.value`           | list<string>      | `parser/templating.rs:get_context` & `set_context_value` | Provides dynamic template variables (`name:value`)                         |
| `templating.hot_reload`      | bool              | `cofg::get` (reload gate), `templating::get_engine`      | Allows disk reload of config / per-request rebuild of Handlebars engine    |
| `toc.path`                   | string (relative) | `markdown.rs:get_toc`, `_make_toc`, `main.rs:index`      | Location (within public) for generated TOC HTML target & base dir for scan |
| `toc.ext`                    | list<string>      | `markdown.rs:get_toc`                                    | File extensions considered for TOC entries                                 |
| `public_path`                | string            | many: `http_ext`, `markdown`, `main`                     | Root directory for content lookup                                          |
| `cache.enable_html`          | bool              | `parser::md2html`                                        | Enable rendered-HTML LRU cache                                             |
| `cache.html_capacity`        | usize             | `parser::md2html`                                        | LRU capacity for HTML cache                                                |
| `cache.enable_toc`           | bool              | `parser::markdown::get_toc`                              | Enable TOC LRU cache                                                       |
| `cache.toc_capacity`         | usize             | `parser::markdown::get_toc`                              | LRU capacity for TOC cache                                                 |

## 顶层配置项映射（Top-Level）

| 字段 (Field)                 | 类型 (Type)       | 代码引用 (Code Reference)                                | 作用 (Effect)                         |
| ---------------------------- | ----------------- | -------------------------------------------------------- | ------------------------------------- |
| `addrs.ip`                   | string            | `main.rs:build_server` via `CofgAddrs`                   | 绑定监听 IP                           |
| `addrs.port`                 | u16               | `main.rs:build_server`                                   | 绑定监听端口                          |
| `tls.enable`                 | bool              | `main.rs:build_server`                                   | 启用 TLS/HTTPS，true 时用 rustls 绑定 |
| `tls.cert`                   | string            | `main.rs:load_tls_config`                                | TLS 证书文件路径（PEM 格式）          |
| `tls.key`                    | string            | `main.rs:load_tls_config`                                | TLS 私钥文件路径（PEM 格式）          |
| `middleware.normalize_path`  | bool              | `main.rs:build_server`                                   | 是否启用路径标准化（Trim）            |
| `middleware.compress`        | bool              | `main.rs:build_server`                                   | 是否启用压缩中间件                    |
| `middleware.logger.enabling` | bool              | `main.rs:build_server`                                   | 启用请求日志                          |
| `middleware.logger.format`   | string            | `main.rs:build_server`                                   | 日志格式字符串，支持自定义 url 替换   |
| `middleware.ratelimit.*`     | 多类型            | `main.rs:build_server`                                   | 启用/配置限流中间件（如速率、窗口等） |
| `middleware.basic_auth.*`    | 多类型            | `main.rs:build_server`                                   | 启用/配置基础认证                     |
| `middleware.ip_filter.*`     | 多类型            | `main.rs:build_server`                                   | 启用/配置 IP 过滤                     |
| `templating.value`           | list<string>      | `parser/templating.rs:get_context` & `set_context_value` | 动态模板变量（name:value DSL）        |
| `templating.hot_reload`      | bool              | `cofg::get` (reload gate), `templating::get_engine`      | 启用热重载，配置/模板每次请求重读     |
| `toc.path`                   | string (relative) | `markdown.rs:get_toc`, `_make_toc`, `main.rs:index`      | TOC 生成目标路径及扫描基准目录        |
| `toc.ext`                    | list<string>      | `markdown.rs:get_toc`                                    | TOC 扫描时纳入的文件扩展名            |
| `public_path`                | string            | 多处：`http_ext`, `markdown`, `main`                     | 内容查找根目录                        |
| `cache.enable_html`          | bool              | `parser::md2html`                                        | 启用 HTML 渲染 LRU 缓存               |
| `cache.html_capacity`        | usize             | `parser::md2html`                                        | HTML 缓存容量                         |
| `cache.enable_toc`           | bool              | `parser::markdown::get_toc`                              | 启用 TOC LRU 缓存                     |
| `cache.toc_capacity`         | usize             | `parser::markdown::get_toc`                              | TOC 缓存容量                          |

WHY: 映射表便于查找配置项影响点，支持快速定位和扩展。

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

## `templating.value` 迷你 DSL

格式：`name:value`

解析顺序：

1. 以第一个 ':' 分割
2. 若 value 以 `env:` 开头，则取环境变量
3. 依次尝试 bool → i64 → 字符串

示例：

```yaml
# cofg.yaml 片段
templating:
  value:
    - "feature_x:true" # 布尔型 => context.bool(feature_x)=true
    - "build:42" # 数字型 => context.number(build)=42
    - "git_hash:env:GIT" # 环境变量查找
    - "title:My Docs" # 字符串
```

## Hot Reload Semantics

| Flag                    | What Triggers Reload                              | Affects                                 |
| ----------------------- | ------------------------------------------------- | --------------------------------------- |
| `templating.hot_reload` | `Cofg::get(true)` calls & every `get_engine` call | Config struct; template engine instance |

中文：若已啟用 hot_reload，程式碼顯式呼叫 `get(true)` 才會重讀設定；模板引擎則每次重新建立，確保檔案修改立即生效。

## 热重载语义（Hot Reload Semantics）

| 标志 (Flag)             | 触发重载条件 (Trigger)                          | 影响对象 (Affects)       |
| ----------------------- | ----------------------------------------------- | ------------------------ |
| `templating.hot_reload` | `Cofg::get(true)` 调用 & 每次 `get_engine` 调用 | 配置结构体、模板引擎实例 |

WHY: 启用 hot_reload 可在开发时实时生效配置和模板变更，生产环境建议关闭。

中文：启用 hot_reload 时，显式调用 `get(true)` 才会重读配置，模板引擎每次请求重建，确保文件修改立即生效。

## Derived Template Variables (Implicit)

| Name             | Source                                                 | Description                               |
| ---------------- | ------------------------------------------------------ | ----------------------------------------- |
| `server-version` | `env!(CARGO_PKG_VERSION)`                              | Crate package version                     |
| `body`           | `md2html` pipeline                                     | Injected rendered HTML fragment           |
| `path`           | Route handlers supply (`path:<req path>` / `path:toc`) | Current logical path for templating logic |

## 隐式派生模板变量（Derived Template Variables, Implicit）

如未显式配置，系统自动注入如下变量：

- `server-version`：当前服务版本号
- `body`：渲染后的 HTML 主体
- `path`：当前请求路径

WHY: 保证模板渲染时有基础上下文，便于扩展和自定义。

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

## Middleware notes

- Order (when enabled): NormalizePath → Compress → Logger → BasicAuth → IP Filter → Handlers
- Rate limiting: configurable via `middleware.rate_limiting.{seconds_per_request, burst_size}`（若專案啟用）

## See also

- Developer guide: ./developer-guide.md
- Request flow: ./request-flow.md
- Performance & caching: ./performance-cache.md
