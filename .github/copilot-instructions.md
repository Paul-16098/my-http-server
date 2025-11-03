<!-- WHY: 精煉給 AI/協作者的高濃度指南；當路由/模板/快取/中介層策略變動時更新。保持約 20–50 行。 -->

## AI 開發速覽（my-http-server）

### 架構與熱路徑

- **架構概述**：
  - `.md` 檔案於執行時轉換為 HTML 片段，並套用 `meta/html-t.hbs`（Handlebars 模板）。
  - 其他副檔名的檔案直接作為靜態檔案回傳。
  - **核心模組**：
    - HTTP：`src/request.rs`、`src/http_ext.rs`、`src/main.rs`
    - 轉換：`src/parser/*`
    - 設定：`src/cofg/*`

### 路由邏輯

- `GET /`：
  - 若存在 `public/index.html`，直接回傳。
  - 否則，使用 `get_toc(public_path)` 產生目錄（TOC），並透過 `md2html` 轉換。
- `GET /{filename:.*}`：
  - 檔案解析至 `public_path`。
  - 若檔案不存在，優先回傳 `meta/404.html`。
  - `.md` 檔案：`read_to_string` + `md2html(path:<rel>)`。
  - 目錄：`get_toc(dir)`（以 `path:toc:<dir>` 渲染）。
  - 其他檔案：`NamedFile::open_async`。

### 模板與 Context

- **模板快取**：
  - `parser/templating::get_engine` 使用 `OnceCell<RwLock<_>>` 快取。
  - 當 `templating.hot_reload=true` 時，每次請求重新建構模板。
- **Context 機制**：
  - 預設包含 `server-version`，渲染時注入 `body`。
  - 支援 `template_data_list` 傳入自定義資料（如 `path:*`）。
  - `templating.value` DSL 支援 `name:value` 與 `name:env:ENV`（型別推斷：bool→i64→string）。
  - 若 `html-t` 未註冊，會自動掛載 `./meta/html-t.hbs`。

### 中介層與安全性

- **中介層功能**：
  - Logger 使用 `%{url}xi`（解碼並去頭 `/`，target=`http-log`）。
  - NormalizePath/Compress 根據旗標啟用。
  - BasicAuth 採用常數時間比較，`allow` 規則優先於 `disallow`。
  - Rate limit：`middleware.rate_limiting.{seconds_per_request,burst_size}`（actix-governor）。
  - IP Filter：`middleware.ip_filter.{enable,allow,block}`（glob 格式，執行於 BasicAuth 之後）。
  - TLS 失敗時回退至 HTTP。
- **注意事項**：尚未強制 canonical prefix，需自行檢查避免 traversal 攻擊。

### 快取與設定

- **全域快取**：
  - `Cofg::new()` 提供全域快取（避免在請求熱路徑強制重讀設定）。
- **每請求快取**：
  - `cached_decoded_uri`、`cached_filename_path`、`cached_public_req_path`、`cached_is_markdown`。
- **跨請求快取**：
  - HTML（LRU）：由 `cache.enable_html` 控制，鍵值包含 `(abs_path,file_mtime,file_size,template_hbs_mtime,template_ctx_hash)`。
  - TOC（LRU）：由 `cache.enable_toc` 控制，鍵值包含 `(dir_abs,dir_mtime,title)`。
  - 設定檔：`cofg.yaml`（預設於 `src/cofg/cofg.yaml`，快取不持久化，重啟後清空）。

### 開發工作流

- **啟動**：
  - 使用 `cargo run` 啟動伺服器。
  - 首次啟動若缺少 `meta/html-t.hbs`，會自動生成預設檔案並退出，需再次執行。
- **測試**：
  - 測試任務：
    - 使用 VS Code 任務「ast-grep: test」進行互動式測試。
    - 使用「ast-grep: scan」快速掃描代碼中的潛在問題。
    - 測試檔案位於 `src/test/*.rs`，覆蓋了核心模組的功能測試。
- **發佈**：
  - 使用 `cargo build --release` 進行發佈。
  - 支援 Docker/Compose（建議容器內使用 `--ip 0.0.0.0`）。
- **CLI 覆寫**：
  - 使用 `build_config_from_cli(Cofg::new(), &cli::Args::parse())` 覆寫設定（如 `--ip/--port`）。

### 常見陷阱與解法

- **模板變更不生效**：
  - 啟用 `templating.hot_reload=true` 或重啟伺服器。
- **舊頁面快取問題**：
  - 確認 `cache.enable_html` 啟用，並檢查鍵值是否包含正確的 mtime/size。
  - TOC 依賴目錄的 mtime，極端情況下需重啟或關閉 `cache.enable_toc`。
- **404 頁面為純文字**：
  - 確認 `meta/404.html` 是否存在。

### 配置示例

- **cofg.yaml 配置**：

  示例：

  ```yaml
  server:
    ip: "127.0.0.1"
    port: 8080
  middleware:
    rate_limiting:
      seconds_per_request: 1
      burst_size: 5
  ```

  使用 `build_config_from_cli` 覆寫 CLI 提供的配置。

### 安全性建議

- **防止目錄遍歷攻擊**：
  - 確保所有路徑都經過正規化處理，避免使用相對路徑（如 `../`）。
  - 僅允許訪問 `public/` 目錄內的文件。
- **中介層安全性**：
  - 啟用 `middleware.ip_filter`，限制允許的 IP 地址範圍。
  - 使用 HTTPS 保護敏感數據傳輸，並配置 TLS 回退機制。

### 快速定位

- **HTTP/路由**：`src/request.rs`、`src/http_ext.rs`、`src/main.rs`
- **轉換邏輯**：`src/parser/{markdown.rs,templating.rs,mod.rs}`
- **設定檔案**：`src/cofg/{config.rs,cofg.yaml}`
- **測試檔案**：`src/test/*.rs`
- **更多細節**：參考 `docs/{request-flow.md, key-functions.md, performance-cache.md, config-templating-map.md}` 與根目錄 `architecture.md`。
