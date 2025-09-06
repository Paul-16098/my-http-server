# AI 專案快速指南 (my-http-server)

> 目的：讓 AI 代理 / Copilot 進入此倉庫後，能在極短時間內正確修改 / 擴充功能，而不做出不合專案慣例的變更。聚焦本 repo 特有的架構、流程與慣例；非通用建議。

## 架構總覽

- 角色：輕量級「Markdown (md/markdown) → HTML」靜態檔案伺服器，基於 `actix-web`。
- 主要模組：
  - `cofg/`：設定載入。`Cofg::new()` 會讀取 (或建立) `./cofg.yaml`，多處呼叫時會重新反序列化；密集迴圈請優先保存一份實例避免重複 IO。
  - `parser/markdown.rs`：批次轉檔與 TOC 生成 (`md2html_all`, `make_toc`)；使用 `wax::Glob` 走訪 `public_path`。
  - `parser/mod.rs`：`md2html`：Markdown AST (markdown-ppp) → HTML，再套用模板引擎 (`mystical-runic`) 的 `html-t.templating`。
  - `parser/templating.rs`：模板引擎/上下文建構：將 `templating.value` 內 `name:value` 字串解析成字串 / 數值 / bool。
  - `main.rs`：流程啟動 → 初始化 logger & 目錄 → 先清除舊 HTML (`remove_public`) → 轉檔 → （可選）產生 TOC → 啟動 HTTP 伺服器（背景 thread）→ 檔案監看 (debounce)。
- 靜態檔案服務：`actix_files::Files::new("/", public_path)`，帶 `index.html` 與自訂 404 (`./meta/404.html`)。
- 模板與靜態資源：預期位於 `./meta` (如 `html-t.templating`)；若新增模板請維持此目錄結構。

## 執行與開發工作流

- 一般開發：`cargo run`（會：初始化 config / 生成 HTML / 啟動 watcher+server）。
- 針對大量 Markdown 變動：可手動呼叫 `parser::markdown::md2html_all()`；保持使用同一份 `Cofg`。
- 交付建置：`cargo build --release`；CI 使用 `houseabsolute/actions-rust-cross` 交叉編譯多平台。
- 測試：目前僅 `src/test/` 內基本測試；新增測試時延續模組內嵌 `#[cfg(test)]` 或集中於此資料夾皆可，但請保持簡潔。

## 重要慣例 / 注意事項

- Config 取得：偏好先 `let cfg = Cofg::new();` 再傳遞引用，避免在迴圈或熱路徑多次呼叫 `Cofg::new()`。
- 轉檔輸出：HTML 檔名 = 原始 Markdown 同路徑改副檔名 `.html`；啟動時會嘗試刪除舊對應 HTML（避免殘留過時內容）。
- TOC：由 `toc.ext` 列表的副檔名（如 html, pdf）掃描組成；結果寫到 `toc.path`；生成時會覆蓋舊檔。
- 檔案監看：`notify` + 1 秒 debounce；期間聚合事件後僅執行一次完整重建。避免在監看回呼內新增阻塞性長任務。
- Logger：`middleware.logger.format` 使用 `%{url}xi` 等佔位；若新增欄位，注意 `actix-web` 支援的格式化鍵值。
- URL 顯示：中介層 logger 會 percent-decode URL；其他邏輯若需 raw URL，請自行取得原值。
- 模板 context 動態值：只允許簡單 `name:value`；複雜結構需在程式端擴充 `get_context`。
- 錯誤處理：目前多以 `unwrap()` / `?`；若改為長期常駐 service，應逐步用 `thiserror` 或自訂錯誤類型（僅在需要時調整，勿一次性大改）。

## 可擴充方向（保持最小侵入）

- 新增自訂模板變數：修改 `templating::get_context` 加入型別判斷；或擴充 `cofg.yaml`。
- 支援更多輸出格式：在 `md2html_all` 中依副檔名條件呼叫不同 render pipeline；保持函式純粹 (input md → output string)。
- 優化性能：將 `Cofg`、`TemplateEngine` 緩存於 `lazy_static` / `once_cell`；確保 hot_reload 模式仍正確失效 cache。

## PR / 變更指引（AI 請遵守）

- 不在未需求驅動下重寫整個模組；專注於需求範圍。
- 若調整公共 API (函式簽名 / 結構欄位)，同步更新使用處與 README 範例。
- 引入新 crate 前：確認標準庫或現有依賴是否已能解決。
- 保持檔案/模組命名簡短小寫；設定檔新增欄位需在 `cofg.yaml` + `Cofg` struct + 預設值同步。

## 範例：新增一個布林模板參數流程

1. 在 `cofg.yaml` 的 `templating.value` 增加 `- "feature_x:true"`。
2. 重新啟動或在 hot reload 模式下直接使用 `{{ feature_x }}` 於 `html-t.templating`。
