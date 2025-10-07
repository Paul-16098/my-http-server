## AI 開發速覽（my-http-server）

目的：用 20–50 行讓代理快速上手本倉庫的結構、熱路徑、與變更守則（避免做壞昂貴路徑）。

1. 這是什麼（Big picture）

- 以 `meta/html-t.templating` 為外殼，將請求到的 `.md` 即時渲染為 HTML；其餘走靜態檔。
- 模組分工：
  - HTTP：`src/request.rs`（路由與處理）+ `src/http_ext.rs`（每請求快取）+ `src/main.rs`（伺服器組裝與中介層）。
  - 轉換：`src/parser/{markdown.rs, templating.rs}`（Markdown→fragment→ 模板引擎）。
  - 設定：`src/cofg/{cofg.rs, cofg.yaml}`（快取 + 可選熱重載）。

2. 路由與處理流程（最常見）

- `GET /`：若有 `public/index.html` 直接回傳；否則 `get_toc(public_path)` 產生 TOC → `md2html`。
- `GET /{filename:.*}`：解析到 `public_path` 下實體路徑；
  - 不存在 → 優先回 `meta/404.html`，否則純文字 404。
  - `.md` → `read_to_string` → `md2html(file, &cfg, vec![format!("path:{}", path)])`。
  - 目錄 → 對該目錄 `get_toc(dir)` 產生清單，並以 `path:toc:<相對目錄>` 渲染。
  - 其他檔案 → `NamedFile::open_async` 靜態回應。

3. 熱路徑與快取（務必遵守）

- 設定：`Cofg::new()` 使用 `OnceCell<RwLock<_>>` 快取；`get(true)` 只有在 `templating.hot_reload=true` 才會重讀磁碟。
- 模板引擎：`get_engine` 首次建立並啟用 bytecode cache；`hot_reload=true` 時每次重建（僅建議於開發）。
- 每請求 Extension 快取：`cached_filename_path`、`cached_public_req_path`、`cached_is_markdown`，避免重複查詢/拼路徑。

4. 模板 Context 規則（`parser/templating.rs`）

- 內建：`server-version`；渲染時注入 `body`；每頁會傳入 `path:*`（如 TOC 用 `path:toc:index`）。
- `templating.value` Mini-DSL：`name:value` 或 `name:env:ENV`；型別推斷順序 bool → i64 → string；缺冒號者忽略。

5. TOC 與路徑處理（`parser/markdown.rs`）

- `get_toc` 以 `toc.ext` 掃描；非英數字元百分比編碼但保留 `/`；Windows `\` 轉 `/`。
- 產生 Markdown 清單後再交由 `md2html` 套上外殼。

6. 開發工作流（本倉庫具體做法）

- 執行：`cargo run`（會自動建立 `public_path`；若缺 `meta/html-t.templating` 會寫入預設並結束，需重跑）。
- 測試：偏好 nextest（VS Code 任務）或 `cargo nextest run --no-fail-fast`；測試在 `src/test/`。
- 發佈：`cargo build --release`；Docker 與 Compose 皆提供（容器中請將 `ip=0.0.0.0`）。
- CLI 覆寫：啟動時以 `build_config_from_cli(Cofg::new(), &cli::Args::parse())` 合成設定（僅支援 `--ip/--port`）。
- 附註：工作區提供 ast-grep 任務（`sg scan` / `sg test --interactive`）可作靜態規則掃描（非必要）。

7. 變更守則（最小侵入）

- 保持 `md2html` 純函式；批次工具請用 `_md2html_all()`（不應接到啟動流程）。
- 熱路徑避免 `Cofg::get(true)`；必要時於測試/管理操作使用。
- 新增 middleware 走 `.wrap(Condition::new(flag, M::new()))` 模式。
- 新增設定欄位需同步：`src/cofg/cofg.yaml`、`Cofg` 結構、`BUILD_COFG` 內嵌預設。

8. 常見陷阱（對應修正）

- 模板改了沒生效：開 `templating.hot_reload=true` 或重啟；TOC/Markdown 永遠即時讀檔。
- 404 顯示純文字：缺 `meta/404.html`。
- 日誌 URL 值：`%{url}xi` 會先 percent-decode；若需原始值改從 `HttpRequest` 取。
- 安全性：尚未強制 canonical prefix；若內容根不受信，請新增前綴檢查避免 traversal。

9. 檔案快速定位

- HTTP/路由：`src/request.rs`、`src/main.rs`、`src/http_ext.rs`
- 轉換/模板：`src/parser/{markdown.rs, templating.rs}`
- 設定：`src/cofg/{cofg.rs, cofg.yaml}`；測試：`src/test/*.rs`

範圍只記載已實作的行為與慣例；細節延伸可參考 `README.md`、`architecture.md`、`request-flow.md`、`key-functions.md`。
