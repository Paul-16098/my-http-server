## AI 開發速覽 (my-http-server)

目標：讓 AI 代理在 <50 行內掌握結構、熱路徑與變更模式，避免誤觸昂貴操作。

### 1. 核心架構

- 角色：即時 Markdown → HTML 包裝（`meta/html-t.templating` 外殼）+ 其餘靜態檔。
- 路由（`src/main.rs`）：
  - `GET /`：若存在 `public/index.html` 直接回傳；否則動態組 TOC ( `get_toc` → `md2html`).
  - `GET /{filename:.*}`：存在? → `.md` 動態 render (`md2html` + 注入 `path:<rel>` )；否則 `NamedFile`；不存在 → 自訂 404。
- 熱路徑函式：`Cofg::new()`（配置快取）、`md2html()`（Markdown→fragment→ 模板 render）、`get_engine()`（模板引擎/可能重建）。

### 2. 配置與 Hot Reload

- `Cofg`：`OnceCell<RwLock<_>>`；`new()` = `get(false)`；只有 hot_reload 且呼叫 `get(true)` 才重讀磁碟。
- 模板引擎：hot_reload=true → 每次重建；否則首次建構 + mystical_runic bytecode cache。
- 常用配置鍵：`public_path`（內容根）、`templating.value`（`name:value` / `name:env:VAR` 注入）、`toc.ext`、middleware toggles。

### 3. 模板 Context 規則 (`parser/templating.rs`)

- 內建：`server-version`, 每頁 `path`，渲染時再寫入 `body`。
- `templating.value` 推斷：bool → i64 → string；缺冒號忽略（安全 no-op）。
- 例：`feature_x:true` → `{{ feature_x }}`；`git_hash:env:GIT` → 從環境變數。

### 4. TOC 與路徑處理

- `get_toc` 走 `public_path`，擴展於 `toc.ext`；非英數 percent-encode，保留 `/`；Windows 轉 `/`。
- Per-request 快取（`http_ext.rs`）提供 decoded uri、是否 markdown、拼接後的 public path，避免重複計算。

### 5. 測試與工作流程

- 執行：`cargo run`（自動建立 `public/` / fallback config）；發佈：`cargo build --release`。
- 測試：偏好 nextest（VS Code task: cargo: nextest）；測試放 `src/test/` 覆蓋 config/templating/markdown/http。
- 新增功能：新增模板變數 → 改 `cofg.yaml` + (dev) 開 `templating.hot_reload`; 新設定欄位 → 同步 `src/cofg/cofg.yaml` + `Cofg` 結構 + `BUILD_COFG`。

### 6. 變更守則（最小侵入）

- 不要將批次轉檔放入啟動流程：維持 `md2html` 純函式；批次用 `_md2html_all()` / `_make_toc()`（僅工具）。
- 避免在熱路徑呼叫 `Cofg::get(true)`；只在管理/測試明確需求下使用。
- 新 middleware：遵循 `.wrap(Condition::new(flag, M::new()))` 模式。

### 7. 常見陷阱 & 快速診斷

- 模板修改無效：確認 `templating.hot_reload=true` 或重啟。
- 404 頁顯示為純文字：檢查 `meta/404.html` 是否存在。
- URL 日誌 encode 異常：格式 `%{url}xi` 會先 percent-decode；需原始值則從 `HttpRequest` 取得。
- 大量重複解析大檔：考慮後續實作 (path, mtime) HTML cache（尚未內建）。

### 8. 安全 / 邊界注意

- 路徑未強制 canonical prefix 檢查（潛在 traversal 改進點）。
- `templating.value` 內容受信；若未來接收使用者輸入需額外 sanitize。

### 9. 快速定位檔案

- Config：`src/cofg/cofg.rs` + `src/cofg/cofg.yaml`
- 轉換/模板：`src/parser/{mod.rs,markdown.rs,templating.rs}`
- HTTP 與進入點：`src/main.rs`
- 測試範例：`src/test/*.rs`

### 10. 新手第一個 PR 建議

新增一個模板變數（含 env 注入）+ 對應測試：確認 context 解析推斷與 hot_reload 生效。

（若需更多：參考 `README.md` 與 `architecture.md`；本檔僅列即刻生產力所需要點。）
