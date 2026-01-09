# Key Functions & Design Rationale

> WHY: 记录关键函数设计意图，降低重构风险，避免无意义复制。中文补充说明便于团队沟通。

> 最后更新时间：2025-12-13
> 适用版本：dev 分支
> 参见：`.github/copilot-instructions.md`（AI 编码代理完整指引）
> 相关：`./developer-guide.md`（常见开发任务）、`./config-templating-map.md`（配置映射）

## Cofg

### `Cofg::new()`

_输入输出：_ 无参数，返回全局配置对象。
_错误处理：_ 初始化失败时抛出 `AppError::Config`。
_性能：_ 仅初始化一次，避免重复加载。
_安全：_ 配置只读，防止运行时篡改。
WHY: 简化热路径调用，防止误用 `force_reload` 导致性能下降。

### `Cofg::get(force_reload)`

_输入输出：_ `force_reload: bool`，返回配置对象。
_错误处理：_ 仅在 `hot_reload` 启用时允许重载，异常时抛 `AppError::Config`。
_性能：_ 生产环境稳定，开发环境支持热重载。
WHY: 保证生产配置稳定，开发可随时刷新。

### `Cofg::force_refresh()`

_输入输出：_ 无参数，强制刷新配置。
_错误处理：_ 仅测试/工具调用，异常抛 `AppError::Config`。
_性能：_ 禁止在运行时调用，防止隐藏开销。
WHY: 仅供测试和工具使用，避免线上性能隐患。

## Templating

### `set_context_value(context, data)`

_输入输出：_ `context` 为模板上下文，`data` 为 `name:value` 字符串。
_错误处理：_ 格式错误（无 ':'）静默忽略，类型推断失败回退为字符串。
_性能：_ 轻量推断，无 schema 爆炸。
_安全：_ 仅允许 bool/i64/string，防止注入复杂类型。
WHY: 灵活扩展模板变量，保证类型可预测。

### `get_context(cfg)`

_输入输出：_ 配置对象，返回全新上下文。
_错误处理：_ 配置异常时抛 `AppError::Template`。
_性能：_ 每次请求独立上下文，避免污染。
WHY: 保证无跨请求污染，简化心智模型。

### `get_engine(cfg)`

_输入输出：_ 配置对象，返回 Handlebars 引擎。
_错误处理：_ 模板编译失败抛 `AppError::Template`。
_性能：_ 热重载时每次重建，开发优先即时性，生产复用提升吞吐。
WHY: 支持开发热重载，生产高性能。

## Markdown & TOC

### `parser_md(input)`

_输入输出：_ 输入 Markdown 字符串，输出 AST。
_错误处理：_ 解析失败抛 `AppError::Parse`。
_性能：_ 封装第三方 crate，便于后续扩展。
WHY: 局部化第三方依赖，便于升级和扩展。

### `get_toc(cfg)`

_输入输出：_ 配置对象，返回 TOC 结构体。
_错误处理：_ 目录遍历异常抛 `AppError::TOC`。
_性能：_ 按需生成，LRU 缓存，键为目录 mtime+title。
_安全：_ 路径百分号编码，防止跨平台分隔符问题。
WHY: 保证目录稳定性和性能。

### `_md2html_all()` / `_make_toc()`

_输入输出：_ 工具函数，批量处理 Markdown/TOC。
_错误处理：_ 仅工具/测试用，异常安全。
_性能：_ 不参与服务启动，保证启动常数时间。
WHY: 工具化批量处理，隔离主流程。

## Rendering Pipeline

### `md2html(md, cfg, extra_vars)`

_输入输出：_ `md` 为 Markdown 字符串，`cfg` 为配置对象，`extra_vars` 为附加上下文变量，返回 HTML 字符串。
_错误处理：_ 解析/渲染异常统一抛 `AppError::Template`，便于 HTTP 响应一致处理。
_性能：_ 支持 HTML 输出缓存，缓存键为 `(abs_path, file_mtime, file_size, template_hbs_mtime, template_ctx_hash)`，仅非 TOC 路径启用。
_安全：_ 变量优先级：请求变量覆盖配置变量，防止污染。
WHY: 串联模板引擎、上下文、AST 解析和渲染，保证灵活性和性能。

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
