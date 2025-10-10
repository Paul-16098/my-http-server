<!--
WHY: 供代理/協作者快速掌握架構與熱路徑，避免在昂貴路徑做出破壞性變更。
維護：當下列情況任一發生時更新本檔：
 - 轉換管線（Markdown→HTML→模板）或模板引擎行為變更
 - 路由/處理流程或快取策略調整
 - `templating.value` Mini-DSL 能力擴充或語義變化
 - 熱重載策略、檔案佈局（meta/public/src）或保留鍵（如 `body`、`server-version`）改動
注意：保持內容短小精悍（20–50 行），細節放到對應源碼註釋與 docs。
-->

## AI 開發速覽（my-http-server）

目的：用 20–50 行讓代理快速上手本倉庫的結構、熱路徑、與變更守則（避免做壞昂貴路徑）。

1. 這是什麼（Big picture）

- 以 `meta/html-t.hbs` 作為頁面外殼；請求到的 `.md` 在執行時轉為 HTML 片段後套用模板，其餘檔案走靜態回應。
- 模組分工：
  - HTTP：`src/request.rs`（兩條路由與主流程）+ `src/http_ext.rs`（每請求快取）+ `src/main.rs`（伺服器組裝）。
  - 轉換：`src/parser/{markdown.rs, templating.rs, mod.rs}`（Markdown→HTML 片段 → 模板渲染）。
  - 設定：`src/cofg/{config.rs, cofg.yaml}`（快取 + 可選熱重載）。

2. 路由與處理流程（最常見）

- `GET /`：若有 `public/index.html` 直接回傳；否則以 `get_toc(public_path)` 動態產生 TOC，再用 `md2html` 套上 `html-t.hbs`。
- `GET /{filename:.*}`：解析到 `public_path` 下實體路徑；
  - 不存在 → 優先回 `meta/404.html`，否則純文字 404。
  - `.md` → `read_to_string` → `md2html(file, &cfg, vec![format!("path:{}", rel_path)])`。
  - 目錄 → `get_toc(dir)` 產生清單，並以 `path:toc:<相對目錄>` 渲染。
  - 其他檔案 → `NamedFile::open_async` 靜態回應。

3. 模板與 Context（Handlebars）

- 模板引擎：`src/parser/templating.rs::get_engine`；預設快取於 `OnceCell<RwLock<_>>`，`templating.hot_reload=true` 時每次重建（開發用）。
- 內建 Context：`server-version`；渲染時會注入 `body`（Markdown HTML 片段）與呼叫端附加的 `template_data_list`（例如 `path:*`）。
- `templating.value` Mini-DSL：`name:value` 或 `name:env:ENV`；型別推斷順序 bool → i64 → string；缺冒號者忽略。
- 模板註冊：邏輯名 `html-t` 對應 `./meta/html-t.hbs`，首次渲染時自動 `register_template_file`。

4. TOC 與路徑處理

- `get_toc(root, cfg, title)` 掃描 `cfg.toc.ext` 副檔名；非英數字元百分比編碼但保留 `/`；Windows `\\` 轉 `/`；可用 `cfg.toc.ig` 字串過濾。
- 產生 Markdown 清單後再交由 `md2html` 套上模板（因此 TOC 也走相同渲染管線）。

5. 熱路徑與快取（請遵守）

- 設定：`Cofg::new()` 使用全域快取；不要在熱路徑呼叫強制重讀。
- 每請求 Extension 快取：`cached_filename_path`、`cached_public_req_path`、`cached_is_markdown`，避免重覆解析。
- 渲染：`md2html` 維持純入口；模板引擎/Context 的副作用已封裝。

6. 開發工作流

- 執行：`cargo run`。
- 測試：`src/test/`；亦可使用工作區任務：`ast-grep: scan` / `ast-grep: test`（靜態掃描，可選）。
- 發佈：`cargo build --release`；Docker/Compose 皆提供（容器內請設 `ip=0.0.0.0`）。
- CLI 覆寫：啟動時以 `build_config_from_cli(Cofg::new(), &cli::Args::parse())` 合成設定（支援 `--ip/--port`）。

7. 常見陷阱（對應修正）

- 模板改了沒生效：開 `templating.hot_reload=true` 或重啟；TOC/Markdown 永遠即時讀檔。
- 404 顯示純文字：缺或讀不到 `meta/404.html`。
- URL 日誌值：`%{url}xi` 會先 percent-decode；若需原始值改從 `HttpRequest` 取。
- 安全性：尚未強制 canonical prefix；若內容根不受信，請新增前綴檢查避免 traversal。

8. 檔案快速定位（Examples）

- HTTP/路由：`src/request.rs`、`src/http_ext.rs`、`src/main.rs`
- 轉換/模板：`src/parser/{markdown.rs, templating.rs, mod.rs}`（`md2html` 定義於 `mod.rs`）
- 設定：`src/cofg/{config.rs, cofg.yaml}`；測試：`src/test/*.rs`
