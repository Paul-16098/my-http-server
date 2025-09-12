# my-http-server

[![Build&release](https://github.com/Paul-16098/my-http-server/actions/workflows/cli.yml/badge.svg?branch=main)](https://github.com/Paul-16098/my-http-server/actions/workflows/cli.yml) [![Security audit](https://github.com/Paul-16098/my-http-server/actions/workflows/Security-audit.yml/badge.svg)](https://github.com/Paul-16098/my-http-server/actions/workflows/Security-audit.yml)

輕量級「Markdown → HTML」靜態伺服器，使用 Rust 與 actix-web。啟動時會將 `public/` 下的 `.md`、`.markdown` 轉成 `.html`，提供靜態服務並支援檔案監看、模板套版與可選 TOC 生成。

## 功能

- 轉檔與靜態服務：啟動時批次將 Markdown 轉為 HTML，並以 actix-files 提供靜態服務（包含目錄清單、可自訂 404）。
- 檔案監看：預設開啟；偵測變更後以 1 秒 debounce 觸發完整重建。
- 模板引擎：以 `meta/html-t.templating` 作為外殼模板；支援 hot reload 與自訂 context 值。
- TOC 生成：可依副檔名清單掃描並輸出到指定路徑（如 `index.html`）。
- 中介層：可選 NormalizePath、Compress、Logger（URL 於日誌中會自動 percent-decode）。

## 快速開始

前置需求：已安裝 Rust（stable）。

1. 取得原始碼並建置

```pwsh
git clone https://github.com/Paul-16098/my-http-server.git
cd my-http-server
cargo build --release
```

2. 準備必要目錄與模板

- 內容目錄：`./public/`
- 模板目錄：`./meta/` 需至少包含：
  - `html-t.templating`：主模板（會注入 `{{ body }}` 與其他 context 變數）
  - `404.html`：自訂 404 頁面

最小範例：

```html
<!-- meta/html-t.templating -->
<!DOCTYPE html>
<html>
  <body>
    {{ body }}
  </body>
</html>
```

```html
<!-- meta/404.html -->
<!DOCTYPE html><meta charset="utf-8" />
<h1>404 Not Found</h1>
```

3. 放一個 Markdown 檔案並啟動伺服器

```pwsh
New-Item -Type Directory -Force public | Out-Null
"# Hello

這是首頁。" | Set-Content -Encoding UTF8 public\index.md

cargo run
```

預設會在 `http://127.0.0.1:8080/` 提供服務。第一次啟動會：

- 清理舊有對應的 HTML（僅刪除 `public/` 下與 md 同名的 `.html`）。
- 將所有 Markdown 轉為 HTML。
- 若 `toc.make_toc=true`，產生 TOC（預設輸出 `public/index.html`）。
- 啟動 HTTP 伺服與檔案監看（可由設定檔關閉）。

## 設定

執行目錄下的 `./cofg.yaml`（可不提供，會使用內建預設值）。目前欄位（對照 `src/cofg/cofg.yaml`）：

- addrs
  - ip: 預設 `127.0.0.1`
  - port: 預設 `8080`
- middleware
  - normalize_path: 是否啟用 NormalizePath（預設 true）
  - compress: 是否啟用 Compress（預設 true）
  - logger
    - enabling: 是否啟用請求日誌（預設 true）
    - format: 日誌格式字串，支援 `%{url}xi`、`%s`、`%{Referer}i`、`%{User-Agent}i` 等
- watch: 是否啟用檔案監看（預設 true）
- templating
  - hot_reload: 是否啟用模板熱重載（預設 true）；啟用時也會允許 `Cofg::get(true)` 重新讀取設定
  - value: 供模板使用的 `name:value` 字串陣列，支援：
    - 布林與數字（i64）
    - 文字（預設）
    - `name:env:ENV_NAME` 會讀取環境變數
- toc
  - make_toc: 是否生成 TOC（預設 true）
  - path: 輸出相對於 `public_path` 的路徑（預設 `index.html`）
  - ext: 需要納入 TOC 掃描的副檔名清單（預設 `html,pdf`）
- public_path: 內容根目錄（預設 `./public/`）

提示：轉檔時每個輸出的頁面會額外注入 `path` 變數（不含 `public_path` 前綴），可在模板中使用；全域還會注入 `server-version`。

## 運作細節

- 移除舊輸出：啟動時會移除 `public/` 內所有 Markdown 對應的 `.html`，避免殘留過期輸出。
- 監看與重建：檔案變更後以 1 秒 debounce，重建「全部」內容，並在需要時重建模板與重載設定（若 `templating.hot_reload=true`）。
- 靜態服務：
  - 根目錄對應 `public_path`，啟用目錄清單與 `index.html`。
  - 404 由 `./meta/404.html` 提供。
- 日誌：URL 會預先 percent-decode 後再寫入日誌。

## 測試

偏好使用 nextest（若未安裝，可直接 `cargo test`）：

```pwsh
cargo nextest run --no-fail-fast
```

測試涵蓋：設定載入/熱更新、模板 context、Markdown 轉 HTML 與 TOC 基本驗證等。測試檔位於 `src/test/`。

## 常見問題（FAQ）

- 啟動失敗：Port 已被占用？請調整 `cofg.yaml` 的 `addrs.port`。
- 空白頁或 404：請確認 `meta/html-t.templating` 與 `meta/404.html` 存在，且 `public/` 有可轉檔的 Markdown 或已有 `index.html`。
- 模板/設定修改未生效：請確認 `templating.hot_reload=true`；否則需重啟程式。
- 自行手動放入的 `.html` 被刪除？啟動時只會刪除「與 Markdown 同名」的 `.html`，避免舊輸出殘留；若與 md 同名則會被清理。

## 專案結構速覽

```tree
src/
   cofg/            # 設定讀取與快取（OnceCell + RwLock）
   parser/          # Markdown 轉 HTML、模板、TOC
   main.rs          # 啟動流程、HTTP 伺服、監看
meta/              # 模板與 404（自行建立）
public/            # 網站內容（自行建立）
```

## 授權與貢獻

歡迎開 Issue 與 PR。提交前請先跑測試以確保通過。
