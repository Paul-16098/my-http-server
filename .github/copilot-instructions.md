<!-- WHY: 精煉給 AI/協作者的高濃度指南；當路由/模板/熱重載/中介層策略變動時更新。保持 20–50 行。 -->

## AI 開發速覽（my-http-server）

目的：讓代理快速掌握架構、熱路徑與約定，避免在昂貴路徑做出破壞性變更。

1. Big picture

- `meta/html-t.hbs` 為外殼；`.md` 於執行時轉 HTML 片段再套模板，其它副檔名走靜態檔。
- 模組：HTTP（`src/request.rs`、`src/http_ext.rs`、`src/main.rs`）／轉換（`src/parser/*`）／設定（`src/cofg/*`）。

2. 路由

- `GET /`：如有 `public/index.html` 直接回傳；否則以 `get_toc(public_path)` 產生 TOC → `md2html`。
- `GET /{filename:.*}`：解析到 `public_path` 下；不存在 → 優先回 `meta/404.html`；`.md`→`read_to_string` + `md2html(path:<rel>)`；目錄 →`get_toc(dir)` 並以 `path:toc:<dir>` 渲染；否則 `NamedFile::open_async`。

3. 模板/Context

- 引擎：`parser/templating::get_engine` 以 `OnceCell<RwLock<_>>` 快取；`templating.hot_reload=true` 時每請求重建。
- Context 內建 `server-version`；渲染時注入 `body`；呼叫端可加 `template_data_list`（如 `path:*`）。
- DSL：`templating.value` 支援 `name:value` 與 `name:env:ENV`（型別推斷：bool → i64 → string）；缺冒號者忽略。
- 模板註冊：邏輯名 `html-t` 對應 `./meta/html-t.hbs`，引擎未註冊時自動註冊。

4. 中介層與安全（`src/main.rs`）

- Logger：`%{url}xi` 先 percent-decode 並去頭 `/`，target=`http-log`。
- NormalizePath/Compress：依旗標啟用；BasicAuth：常數時間比較，`allow` 先於 `disallow`，未設定使用者則預設拒絕。
- Rate Limiting：`middleware.rate_limiting.{seconds_per_request, burst_size}`（actix-governor）。
- IP Filter：`middleware.ip_filter.{enable,allow,block}`（glob）；序位於 BasicAuth 之後；停用為零開銷。
- TLS：`tls.enable` 載入 `cert/key` 建立 rustls；失敗會回退為 HTTP。
- Security：尚未強制 canonical prefix；不可信內容根時需自行加前綴檢查避免 traversal。

5. 熱路徑/快取

- 設定：`Cofg::new()` 走全域快取；勿在請求熱路徑強制重讀。
- 每請求快取鍵：`cached_filename_path`、`cached_public_req_path`、`cached_is_markdown`。
- 渲染入口：維持 `md2html` 為唯一入口（模板/Context 副作用已封裝）。
- 交叉請求快取：
  - HTML 快取（LRU）：啟用由 `cache.enable_html` 控制；容量 `cache.html_capacity`。命中鍵 = `(abs_path, file_mtime, file_size, template_hbs_mtime, template_ctx_hash)`。
  - 使用限制：僅當 Context 含 `path:<rel>` 且非 `path:toc:*` 才會啟用 HTML 快取。
  - TOC 快取（LRU）：啟用由 `cache.enable_toc` 控制；容量 `cache.toc_capacity`。命中鍵 = `(dir_abs, dir_mtime, title)`。
  - 設定位置：`cofg.yaml`（內建預設於 `src/cofg/cofg.yaml`）。無持久化；重啟即清空。

6. 工作流/陷阱

- 執行：`cargo run`；首次若缺 `meta/html-t.hbs` 會寫入預設後退出，需再執行一次。
- 測試：`src/test/*.rs`；可用任務：`ast-grep: scan` / `ast-grep: test`；發佈：`cargo build --release`（Docker/Compose 可用，容器內設 `ip=0.0.0.0`）。
- CLI 覆寫：`build_config_from_cli(Cofg::new(), &cli::Args::parse())`（如 `--ip/--port`）。
- FAQ：模板不生效 → 開 `templating.hot_reload=true` 或重啟；404 純文字 → 缺 `meta/404.html`；BasicAuth 以 `allow`/`disallow` 判序；日誌 URL 已解碼。
  - 快取相關：變更 Markdown/模板後仍見舊頁 → 檢查 `cache.enable_html` 是否開啟；HTML 快取鍵含 mtime/size 與模板 mtime，正常會自動失效。TOC 依賴目錄 mtime，極端檔案系統行為需重啟或關閉 `cache.enable_toc`。

7. 快速定位

- HTTP/路由：`src/request.rs`、`src/http_ext.rs`、`src/main.rs`；轉換：`src/parser/{markdown.rs, templating.rs, mod.rs}`；設定：`src/cofg/{config.rs, cofg.yaml}`；測試：`src/test/*.rs`。
