# AI 專案快速指南 (my-http-server)

> 目的：讓 AI 代理能立刻在此 Rust/actix-web 專案中安全地擴充功能或修正問題；聚焦本 repo 的實作與慣例。

## 架構速覽（大圖）

- 角色：輕量級「Markdown → HTML」靜態伺服器，轉檔後由 `actix-files` 提供靜態服務。
- 關鍵模組與檔案：
  - `src/cofg/`：設定讀取與全域快取。`Cofg::get(force_reload)` 以 `OnceCell<RwLock<_>>` 快取；`force_reload=true` 且 `templating.hot_reload=true` 時會重讀 `./cofg.yaml`。預設值來自編譯內嵌 `cofg.yaml`（`BUILD_COFG`）。
  - `src/parser/markdown.rs`：批次轉檔 `md2html_all()`；TOC 生成 `make_toc()`；透過 `wax::Glob` 掃描 `public_path`。
  - `src/parser/mod.rs`：`md2html(md, &Cofg)` 將 Markdown AST（markdown-ppp）轉成 HTML，再用模板 `./meta/html-t.templating` 包裝。
  - `src/parser/templating.rs`：`TemplateEngine` 與 `TemplateContext`。引擎以 `OnceCell` 快取；`hot_reload` 時會重建以反映模板變更。Context 解析 `templating.value` 的 `name:value` 字串為 bool/number/string，並預設注入 `server-version`。
  - `src/main.rs`：啟動流程：init logger/目錄 → `remove_public`（刪除既有對應 HTML）→ `md2html_all` → `make_toc?` → 啟動 HTTP 伺服（背景 thread）→ 檔案監看（`notify`，1 秒 debounce，忽略 `.git`）。
- 靜態檔案服務：`actix_files::Files::new("/", public_path)`；`index_file("index.html")`；自訂 404 來自 `./meta/404.html`；可清單目錄。

## 開發與測試工作流

- 建置/執行：`cargo run`（會初始化 config、轉檔並啟動 watcher+server）。發佈：`cargo build --release`。
- 測試：測試位於 `src/test/`。VS Code 內建工作「cargo: nextest」可執行；若未安裝 nextest，使用 `cargo test`。
- 內容目錄：預設 `public_path: ./public/`；模板放在 `./meta/`（需包含 `html-t.templating`、`404.html`）。

### 測試策略（擴充到更多模組）

- 位置：維持 `src/test/` 作為整合/模組測試；輕量單元測試可就地放在各模組 `#[cfg(test)]`。
- parser：
  - `md2html`：以最小 Markdown（如 `# h1`）驗證輸出包含 `<h1>` 且套上模板殼（context 中 `server-version` 可檢查）。
  - `md2html_all/make_toc`：使用臨時資料夾（`std::env::temp_dir()` + 手動清理）建立 `public/` 與 `meta/html-t.templating`，生成後斷言輸出檔存在與基本內容。
- templating：對 `get_context` 餵入 `value: ["a:true","b:1","c:txt"]`，斷言 bool/number/string 三型皆正確。
- cofg：修改暫存 `cofg.yaml` 後呼叫 `Cofg::get(true)`，在 `hot_reload=true` 下斷言值有更新；也測 `Default` 與 `CofgAddrs` Display（範例如現有 `src/test/cofg.rs`）。
- HTTP（選配）：用 `actix_web::test` 啟動與 `run_server` 相同的 `App` 組態，對不存在路徑斷言 404 來自 `meta/404.html`；靜態清單可驗證索引。
- 監看（選配/慢）：避免直接測 `notify` 事件；僅驗證回呼核心邏輯（呼叫 `md2html_all` + `make_toc?`）。

## CI / 發佈

- 安全稽核：`.github/workflows/Security-audit.yml` 每週與依賴變更時跑 `rustsec/audit-check`。
- 建置/發佈：`.github/workflows/cli.yml` 透過 `houseabsolute/actions-rust-cross` 交叉編譯多平台並以 `actions-rust-release` 發佈產物；在 push/PR/手動觸發時執行。

## 專案慣例與注意事項

- 設定取得：在熱路徑請重用 `let cfg = Cofg::get(false);` 並傳參考；只有在需要讀入最新檔案時（且 `hot_reload=true`）才用 `Cofg::get(true)`。
- 轉檔規則：輸出 HTML 檔名 = 原始 Markdown 檔改副檔名 `.html`；啟動時 `remove_public()` 會清掉舊對應 HTML，避免殘留。
- TOC：根據 `toc.ext`（如 `html,pdf`）掃描，寫到 `toc.path`（相對於 `public_path`）。生成會覆蓋舊檔。
- 檔案監看：`notify` + 1 秒 debounce；回呼僅觸發一次完整重建（`md2html_all` + `make_toc?`）。避免在回呼中加入長時間阻塞任務。
- Logger 與 URL：中介層 logger 使用 `%{url}xi` 等佔位，並對 URL 做 percent-decode；若邏輯需要 raw URL，請自行從請求取得原值。
- 模板 context：只支援簡單 `name:value`；複雜結構請擴充 `get_context`。`TemplateEngine` 會在 `hot_reload` 下自動重建。

## 擴充與變更指引（小改動優先）

- 加模板變數：在 `cofg.yaml` 的 `templating.value` 增加條目（例如 `- "feature_x:true"`），模板可直接用 `{{ feature_x }}`；`hot_reload=true` 可免重啟。
- 新設定欄位：同時更新 `src/cofg/cofg.yaml`、`Cofg` 結構（`nest_struct`）、並確保 `Default`（`BUILD_COFG`）同步。
- 新輸出格式：在 `md2html_all()` 依副檔名分支呼叫對應 render；維持 `md2html(md, &Cofg) -> String` 的純函式特性。

## 參考與位置

- 設定範例：`src/cofg/cofg.yaml`
- 模板：`meta/html-t.templating`、`meta/404.html`
- 核心流程：`src/main.rs`、`src/parser/markdown.rs`、`src/parser/mod.rs`、`src/parser/templating.rs`

有未盡之處或需補充的開發流程（例如特定 CI/發佈規範）請回饋，我會再精煉補齊。
