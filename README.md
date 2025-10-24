# My HTTP Server

## Project Name and Description

My HTTP Server 是一個用於處理靜態檔案與 Markdown 檔案的高效能 HTTP 伺服器。其主要功能包括：

- 提供靜態檔案服務
- 將 Markdown 檔案轉換為 HTML
- 支援模板渲染與快取機制

## Technology Stack

- **語言**: Rust

- **框架與函式庫**:
  - Actix-web: 提供 HTTP 伺服器功能
  - Handlebars: 用於模板渲染
  - OnceCell: 實現快取
  - Actix-governor: 實現速率限制

## Project Architecture

- **架構概述**:

  - `.md` 檔案於執行時轉換為 HTML 片段，並套用 `meta/html-t.hbs`（Handlebars 模板）。
  - 其他副檔名的檔案直接作為靜態檔案回傳。

- **核心模組**:
  - HTTP: `src/request.rs`、`src/http_ext.rs`、`src/main.rs`
  - 轉換: `src/parser/*`
  - 設定: `src/cofg/*`

## Getting Started

### 安裝與啟動

1. 確保已安裝 Rust 環境。

2. 使用以下指令啟動伺服器：

   ```bash
   cargo run
   ```

3. 若首次啟動缺少 `meta/html-t.hbs`，伺服器會自動生成預設檔案並退出，需再次執行。

### 測試

- 測試檔案位於 `src/test/*.rs`。

## Project Structure

- `src/`: 核心程式碼

  - `request.rs`, `http_ext.rs`, `main.rs`: HTTP 處理邏輯
  - `parser/`: Markdown 與模板處理
  - `cofg/`: 設定管理

- `meta/`: 預設模板與錯誤頁面

- `public/`: 靜態檔案目錄

- `docs/`: 文件與開發指南

## Key Features

- 支援靜態檔案與 Markdown 檔案服務
- 模板渲染與快取機制
- 基於 Actix-governor 的速率限制
- 支援 IP 過濾與 BasicAuth 驗證

## Development Workflow

- **分支策略**: 使用 `dev` 作為預設開發分支。

- **開發流程**:

  1. 啟動伺服器進行開發。
  2. 使用 VS Code 任務進行程式碼掃描與測試。
  3. 發佈時使用 `cargo build --release`。

## Coding Standards

- 遵循 Rust 社群最佳實踐。
- 使用 `clippy` 進行靜態程式碼分析。

## Testing

- 測試檔案位於 `src/test/*.rs`。
- 使用 Rust 的內建測試框架進行單元測試。

## Contributing

- 提交程式碼前請確保通過所有測試。
- 參考 `src/test/*.rs` 中的程式碼範例進行測試。
- 如需更多貢獻指南，請參考 `docs/developer-guide.md`。
