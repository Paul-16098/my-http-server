# my-http-server

[![Build&release](https://github.com/Paul-16098/my-http-server/actions/workflows/cli.yml/badge.svg?branch=main)](https://github.com/Paul-16098/my-http-server/actions/workflows/cli.yml) [![Security audit](https://github.com/Paul-16098/my-http-server/actions/workflows/Security-audit.yml/badge.svg)](https://github.com/Paul-16098/my-http-server/actions/workflows/Security-audit.yml) [![Docker](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-publish.yml) [![Docker-test](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-test.yml/badge.svg?branch=dev)](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-test.yml)

輕量級 Markdown → HTML 伺服器（Rust + Actix）：對 `.md` 檔動態轉成 HTML 片段並套用 Handlebars 模板（`meta/html-t.hbs`），其他副檔名直接以靜態檔回應。首頁若無 `public/index.html`，會掃描 `public/` 產生 TOC 後再渲染。設定具全域快取，並支援可選模板熱重載。

## Technology Stack

- Language: Rust (edition 2024) · Crate version: 3.2.0（Cargo.toml）
- Web: actix-web 4.11.0 · actix-files 0.6.8
- Security: actix-ip-filter 0.3.2 · actix-governor 0.10.0（速率限制，可選）
- TLS: rustls 0.23 · rustls-pki-types 1.12.0（actix-web features: rustls-0_23）
- Templating: Handlebars 6.3.2（支援 hot reload）
- Markdown: markdown-ppp 2.7.1（AST → HTML fragment）
- Config & Utils: config 0.15.x · once_cell 1.x · serde/serde_json 1.x · clap 4.5.x · env_logger 0.11.x · log 0.4.x · percent-encoding 2.3.x · wax 0.6.x · thiserror 2.x · nom 8.x · nest_struct 0.5.x · lru 0.16.x

> 來源：Cargo.toml 與 `.github/copilot-instructions.md`

## Project Architecture

- HTTP/路由：`src/request.rs`（`GET /` 與 `GET /{filename:.*}`）；`src/main.rs`（伺服器組裝與中介層）；`src/http_ext.rs`（每請求衍生值快取）
- 內容轉換：`src/parser/{markdown.rs, templating.rs, mod.rs}`（Markdown → HTML 片段 → 套模板）
- 設定：`src/cofg/{config.rs, cofg.yaml}`（全域快取；支援可選熱重載）

動態 Markdown 流程（高層）：

```text
HTTP Request → 路由 → 解析 public 實體路徑 → .md?
  是：read_to_string → md2html（AST→HTML 片段 → Handlebars 渲染）→ 200
  否：NamedFile::open_async（靜態檔） → 200；不存在：偏好 meta/404.html
```

首頁（`/`）：若 `public/index.html` 存在則直接回傳；否則以 `get_toc(public_path)` 產生 TOC 後走 `md2html` 渲染。

更多細節請見：`architecture.md`、`docs/request-flow.md`、`docs/key-functions.md`、`docs/performance-cache.md`。

## Getting Started

> 以下指令為文件用途（選擇性）

• 開發啟動（首次缺 `meta/html-t.hbs` 會寫入預設後退出；請再執行一次）

```pwsh
cargo run
```

• 預設位址：`http://127.0.0.1:8080/`

• 設定檔：`cofg.yaml`（若缺會自動由內嵌預設建立）。樣板見 `src/cofg/cofg.yaml`；常用欄位如 `public_path`、`templating.{value,hot_reload}`、`cache.*`、`middleware.*`、`tls.*`。

### Optional: Docker/Compose

```pwsh
docker build -t my-http-server .
docker run --rm -p 8080:8080 `
  -v ${PWD}/public:/app/public `
  -v ${PWD}/cofg.yaml:/app/cofg.yaml `
  my-http-server
```

TLS 範例：

```pwsh
docker run --rm -p 8443:8443 `
  -v ${PWD}/public:/app/public `
  -v ${PWD}/cofg.yaml:/app/cofg.yaml `
  -v ${PWD}/cert.pem:/app/cert.pem:ro `
  -v ${PWD}/key.pem:/app/key.pem:ro `
  my-http-server --ip 0.0.0.0 --port 8443 --tls-cert /app/cert.pem --tls-key /app/key.pem
```

或使用 Compose：

```pwsh
docker compose up -d --build
```

## Project Structure

```text
src/
  cofg/        # 設定載入與全域快取（OnceCell + RwLock）
  parser/      # Markdown 解析、模板、TOC 相關
  request.rs   # 路由與處理流程（/ 與 /{filename:.*}）
  http_ext.rs  # 每請求 Extension 衍生值快取（路徑/副檔名等）
  main.rs      # 伺服器組裝與中介層
meta/          # Handlebars 模板與 404 頁面（本倉庫已提供樣板）
public/        # 網站內容（本倉庫提供範例，可自行替換）
docs/          # 架構、流程、效能、IP filter 等說明
```

參考：`docs/config-templating-map.md`（設定欄位對應程式與模板）。

## Key Features

- 即時 Markdown → HTML：`md2html` 管線產出 HTML 片段並套用 `meta/html-t.hbs`
- 首頁自動 TOC：`/` 無 `index.html` 時掃描 `public/`（依 `toc.ext`）並渲染
- 每請求衍生值快取：`decoded_uri`、`filename_path`、`public_req_path`、`is_markdown`
- 模板熱重載（可選）：`templating.hot_reload=true` 時每請求重建引擎；設定強制重讀需顯式 `Cofg::get(true)`
- 靜態檔快速回應：非 `.md` 直接 `NamedFile::open_async`
- 自訂 404：支援 `meta/404.html`
- 安全與中介層：BasicAuth、IP Filter（glob）、Rate limit（actix-governor）、NormalizePath、Compress、Logger
- 快取：HTML（跨請求 LRU，可由 `cache.enable_html` 控制）、TOC（跨請求 LRU，可由 `cache.enable_toc` 控制）

## Development Workflow

- 預設協作分支：`dev`（請自 `dev` 建立功能分支）
- 常用命令（文件用途，選擇性）：

```pwsh
cargo build --release
cargo test
cargo nextest run --no-fail-fast
```

- PR 建議：撰寫/更新測試；執行 `cargo clippy -- -D warnings`；在 PR 描述中說明動機並參照本文段落

> 來源：`docs/developer-guide.md`、`.github/copilot-instructions.md`

## Coding Standards

- `md2html` 保持純粹（無全域副作用）；模板引擎/Context 變化封裝於 `parser/templating.rs`
- 請勿在熱路徑強制設定重讀：避免於請求流程呼叫 `Cofg::get(true)`
- 新增中介層以 `.wrap(Condition::new(flag, M::new()))` 方式組裝於 `main.rs`
- 新增設定需同步三處：`src/cofg/cofg.yaml`、`Cofg` 結構、內嵌預設（BUILD_COFG）
- 安全性：目前未強制 canonical prefix；若內容根不可信，請自行做前綴檢查避免 traversal

> 來源：`.github/copilot-instructions.md`、`docs/*`

## Testing

- 範圍：設定載入/熱重載、模板 Context、Markdown→HTML、TOC 生成、HTTP 行為
- 位置：`src/test/*.rs`
- 執行（文件用途，選擇性）：

```pwsh
cargo nextest run --no-fail-fast
```

工作區亦提供 VS Code 任務：`ast-grep: scan` 與 `ast-grep: test`。

## Contributing

- 從 `dev` 建立功能分支並發送 PR
- 參考實作與風格：`docs/key-functions.md`、`docs/developer-guide.md`、`.github/copilot-instructions.md`
- 提交前：補齊與更新測試、`cargo clippy -- -D warnings`、確保所有測試通過

—

參考文件：

- `architecture.md` — 系統架構與資料流
- `docs/request-flow.md` — 路由流程與時序圖
- `docs/key-functions.md` — 關鍵函式與設計理由
- `docs/performance-cache.md` — 效能與快取筆記
