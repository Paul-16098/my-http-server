# AI 開發速覽 (my-http-server)

目的：讓 AI 代理在此 Rust/actix-web 專案中快速上手，並遵循本庫既有模式與命名。

## 架構與路由

- 角色：Markdown → HTML 的輕量伺服器；.md 以「即時渲染」回應，其餘走靜態檔。
- 主要檔案/模組：
  - 設定：`src/cofg/cofg.rs`（`Cofg` 以 `OnceCell<RwLock<_>>` 快取；`Cofg::new()` 等同 `get(false)`；`get(true)` 僅在 `templating.hot_reload=true` 時從磁碟重讀；預設值 `BUILD_COFG` 來自內嵌 `src/cofg/cofg.yaml`）。
  - 模板：`src/parser/templating.rs`（`get_engine(&Cofg)` 回傳 `mystical_runic::TemplateEngine`，在 hot_reload 時重建；`get_context(&Cofg)` 預設注入 `server-version`，並把 `templating.value` 的 `name:value` 轉成 bool/number/string；支援 `name:env:ENV`）。
  - 轉換：`src/parser/mod.rs::md2html(md, &Cofg, Vec<String>) -> AppResult<String>` 先以 markdown-ppp 產 HTML，再用 `meta/html-t.templating` 包裝並注入 `{{ body }}`。
  - TOC：`src/parser/markdown.rs::get_toc(&Cofg)` 掃描 `public_path` 下符合 `toc.ext` 的檔名並產生 Markdown 連結清單；`_md2html_all()`、`_make_toc()` 是工具函式，非啟動流程一部分。
  - HTTP：`src/main.rs` 定義兩條路由：`GET /` 若 `public/index.html` 存在則直接回傳，否則以 `get_toc` 即時產 TOC 再 `md2html`；`GET /{filename:.*}` 對 .md 即時渲染，其餘走 `actix_files::NamedFile`；404 讀取 `meta/404.html`。

## 開發與測試

- 執行：`cargo run`（初始化 logger、建立 `public/` 目錄；不含檔案監看或啟動前批次轉檔）。發佈：`cargo build --release`。
- 目錄假設：內容根目錄 `public/`，模板位於 `meta/`，至少需 `html-t.templating` 與 `404.html`。
- 測試：偏好 nextest。VS Code 已提供工作「cargo: nextest」。亦可用 `cargo test`。測試放在 `src/test/`（含 cofg/templating/parser 的行為）。

## 專案慣例與細節

- 取得設定：在熱路徑使用 `Cofg::new()` 取得快取；只有在需要同步外部變更時才呼叫 `Cofg::get(true)`。
- 模板資料：常見注入為 `vec![format!("path:{}", <相對於 public 的路徑>)]`，例如 `main.rs` 中將請求檔案 path 帶入模板變數 `path`。
- Logger URL：請求日誌格式支援 `%{url}xi`，實作會先 percent-decode；若需要原始 URL，請從 `HttpRequest` 自行取值。
- TOC 連結：以 percent-encoding 處理非英數，但保留 `/`；Windows 路徑會轉為 `/`。
- 模板引擎：hot_reload=true 時每次取用會重建引擎；否則沿用 OnceCell 快取並啟用 bytecode cache。

## 常見擴充（保守變更優先）

- 新的模板變數：在 `cofg.yaml` 的 `templating.value` 增加（如 `- "feature_x:true"`），模板以 `{{ feature_x }}` 使用；hot_reload 可即時生效。
- 新設定欄位：同步更新 `src/cofg/cofg.yaml` 與 `Cofg`（`nest_struct` 巨集會生成子結構）；確保 `Default` 與 `BUILD_COFG` 一致。
- 新輸出樣式：維持 `md2html` 純函式；需要批次轉檔時，新增工具命令而非修改啟動流程。

## 參考節點

- 設定：`src/cofg/cofg.rs`、`src/cofg/cofg.yaml`
- 模板：`meta/html-t.templating`、`meta/404.html`
- 轉換：`src/parser/mod.rs`、`src/parser/markdown.rs`、`src/parser/templating.rs`
- HTTP：`src/main.rs`

若有不清楚或待補的流程（例如是否需要新增 watcher、批次轉檔命令介面），請留言，我會再補齊或調整本指南。
