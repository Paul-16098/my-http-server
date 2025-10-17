<!-- WHY: 精煉給 AI/協作者的高濃度指南；當路由/模板/快取/中介層策略變動時更新。保持約 20–50 行。 -->

## AI 開發速覽（my-http-server）

- 架構與熱路徑：`.md` 於執行時 → HTML 片段 → 套 `meta/html-t.hbs`（Handlebars）；其他副檔名直送靜態檔。核心模組：HTTP（`src/request.rs`、`src/http_ext.rs`、`src/main.rs`）／轉換（`src/parser/*`）／設定（`src/cofg/*`）。
- 路由：
  - `GET /`：若有 `public/index.html` 直接回傳；否則以 `get_toc(public_path)` 產生 TOC 後走 `md2html`。
  - `GET /{filename:.*}`：解析到 `public_path`；不存在 → 偏好 `meta/404.html`；`.md`→`read_to_string` + `md2html(path:<rel>)`；目錄 →`get_toc(dir)`（以 `path:toc:<dir>` 渲染）；其餘 `NamedFile::open_async`。
- 模板/Context：`parser/templating::get_engine` 以 `OnceCell<RwLock<_>>` 快取；`templating.hot_reload=true` 時每請求重建。Context 內建 `server-version`，渲染時注入 `body`；可經 `template_data_list` 傳入 `path:*` 等。`templating.value` DSL 支援 `name:value` 與 `name:env:ENV`（型別推斷：bool→i64→string）。`html-t` 未註冊時自動掛載 `./meta/html-t.hbs`。
- 中介層/安全（`src/main.rs`）：Logger 用 `%{url}xi`（先解碼並去頭 `/`，target=`http-log`）；NormalizePath/Compress 依旗標；BasicAuth 常數時間比較，`allow` 先於 `disallow`，未設定使用者預設拒絕；Rate limit 由 `middleware.rate_limiting.{seconds_per_request,burst_size}`（actix-governor）；IP Filter `middleware.ip_filter.{enable,allow,block}`（glob，序位於 BasicAuth 之後）；TLS 失敗回退 HTTP；尚未強制 canonical prefix（不可信根需自行做前綴檢查以避免 traversal）。
- 設定/快取：`Cofg::new()` 全域快取（勿在請求熱路徑強制重讀）。每請求快取：`cached_decoded_uri`、`cached_filename_path`、`cached_public_req_path`、`cached_is_markdown`。跨請求快取：
  - HTML（LRU）：由 `cache.enable_html` 控制，容量 `cache.html_capacity`；鍵 `(abs_path,file_mtime,file_size,template_hbs_mtime,template_ctx_hash)`；僅當 Context 含 `path:<rel>` 且非 `path:toc:*` 才啟用。
  - TOC（LRU）：由 `cache.enable_toc` 控制，容量 `cache.toc_capacity`；鍵 `(dir_abs,dir_mtime,title)`。
  - 設定檔：`cofg.yaml`（內建預設於 `src/cofg/cofg.yaml`），快取不持久化，重啟清空。
- 開發工作流：
  - 啟動：`cargo run`（首啟若缺 `meta/html-t.hbs` 會寫入預設後退出；請再執行一次）。
  - 測試：`src/test/*.rs`；可用工作：VS Code 任務「ast-grep: scan / test」。
  - 發佈：`cargo build --release`；Docker/Compose 可用（容器內建議 `--ip 0.0.0.0`）。
  - CLI 覆寫：`build_config_from_cli(Cofg::new(), &cli::Args::parse())`（如 `--ip/--port`）。
- 常見陷阱：
  - 模板變更不生效 → 開 `templating.hot_reload=true` 或重啟。
  - 看到舊頁 → 確認 `cache.enable_html`、鍵已含 mtime/size 與模板 mtime；TOC 依賴目錄 mtime，極端情況需重啟或關閉 `cache.enable_toc`。
  - 404 純文字 → 缺 `meta/404.html`。
- 快速定位：HTTP/路由 →`src/request.rs`、`src/http_ext.rs`、`src/main.rs`；轉換 →`src/parser/{markdown.rs,templating.rs,mod.rs}`；設定 →`src/cofg/{config.rs,cofg.yaml}`；測試 →`src/test/*.rs`。更多細節見 `docs/{request-flow.md, key-functions.md, performance-cache.md, config-templating-map.md}` 與根目錄 `architecture.md`。
