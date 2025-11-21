# my-http-server 文件索引 / Documentation Index

> 快速導覽專案文件，並對齊目前的路由、模板、與中介層行為。

## 快速開始 / Quick start

1. 開發執行：`cargo run`
2. 首次啟動若缺 `meta/html-t.hbs` 或 `cofg.yaml`，程式會寫入預設檔後退出；請再次執行。

相關：

- 根目錄《architecture.md》：整體架構與模組切面
- 根目錄《README.md》：功能摘要與使用說明

## 文件地圖 / Docs map

- Request 與流程

  - [Request Flow & Sequence](./request-flow.md)
  - [Key Functions & Design Rationale](./key-functions.md)

- 模板與設定

  - [Config ↔ Template & Code Usage Map](./config-templating-map.md)
  - [Developer Guide（深入開發指引）](./developer-guide.md)

- 效能與快取

  - [Performance & Caching Notes](./performance-cache.md)

- 安全與中介層
  - [IP Filter 功能說明](./ip-filter.md)
  - [IP Filter 實作摘要](./ip-filter-implementation-summary.md)

## 常見重點 / Key behaviors

- 路由：

  - `GET /`：若存在 `public/index.html` 直接回傳；否則產生 TOC 並以模板渲染。
  - `GET /{filename:.*}`：解析至 `public_path` 下；`.md` 以 `md2html` 渲染；目錄回 TOC；其餘為靜態檔；不存在則偏好 `meta/404.html`。

- 模板：

  - `templating.value` 以 `name:value`/`name:env:ENV` DSL 注入 Context（型別推斷：bool→i64→string）；`templating.hot_reload=true` 時每請求重建模板引擎（Handlebars）。

- 中介層順序（啟用時）：NormalizePath → Compress → Logger → BasicAuth → IP Filter → 處理器。

- 快取：
  - 全域：設定（Cofg）與模板引擎（Handlebars）。
  - 每請求：路徑衍生值（decoded_uri、filename_path、public_req_path、is_markdown）。
  - 跨請求：
    - HTML（Markdown → 完整頁面）：LRU，鍵包含 `(abs_path, file_mtime, file_size, template_hbs_mtime, template_ctx_hash)`；可由 `cache.enable_html` 關閉。
    - TOC：LRU，鍵包含 `(dir_abs, dir_mtime, title)`；可由 `cache.enable_toc` 關閉。

## 導讀建議

1. 先讀《request-flow.md》了解熱路徑
2. 依需求深入《config-templating-map.md》《key-functions.md》
3. 針對效能關注《performance-cache.md》
4. 若需 IP 來源控管，參考《ip-filter.md》《ip-filter-implementation-summary.md》

## 參考 / References

- 上層《../architecture.md》
- 上層《../README.md》
