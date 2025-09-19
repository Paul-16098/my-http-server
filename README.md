# my-http-server

[![Build&release](https://github.com/Paul-16098/my-http-server/actions/workflows/cli.yml/badge.svg?branch=main)](https://github.com/Paul-16098/my-http-server/actions/workflows/cli.yml) [![Security audit](https://github.com/Paul-16098/my-http-server/actions/workflows/Security-audit.yml/badge.svg)](https://github.com/Paul-16098/my-http-server/actions/workflows/Security-audit.yml)[![Docker](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-publish.yml)[![Docker-test](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-test.yml/badge.svg?branch=dev)](https://github.com/Paul-16098/my-http-server/actions/workflows/docker-test.yml)

輕量級「Markdown → HTML」伺服器，使用 Rust 與 actix-web。支援：

- 即時渲染：請求 `.md` 路徑時現場轉成 HTML 回應（使用 `meta/html-t.templating` 外殼）。
- 自訂模板與 Context：hot reload、簡單 `name:value` 型別（含環境變數注入）。
- 404 與中介層：自訂 `meta/404.html`、NormalizePath、Compress、Logger（URL 於日誌中會先 percent-decode）。

## 功能

- 即時 Markdown → HTML：對 `.md` 請求直接回傳渲染結果。
- 批次轉檔（監看時）：變更觸發 `md2html_all()` 與 `make_toc()`（若啟用）。
- TOC 與首頁：
  - `GET /` 若有 `public/index.html` 則直接回傳；否則即時產出 TOC（依 `toc.ext`）。
- 模板引擎：`meta/html-t.templating` 為外殼；預設注入 `server-version` 與每頁 `path`。
- 中介層：NormalizePath、Compress、Logger（可依設定啟用）。

## 快速開始

需求：Rust（stable）。

1. 準備模板與內容目錄（最小化範例）

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

1. 放一個 Markdown 並啟動

```pwsh
New-Item -Type Directory -Force public,meta | Out-Null
"# Hello\n\n這是首頁。" | Set-Content -Encoding UTF8 public\index.md

cargo run
```

預設服務位置：`http://127.0.0.1:8080/`。

## 設定（cofg.yaml）

在執行目錄下放 `./cofg.yaml`（可省略，程式會以編譯內嵌預設值啟動，且首次啟動會落地一份）。重點欄位（對照 `src/cofg/cofg.yaml`）：

- addrs
  - ip：預設 `127.0.0.1`（容器環境請改 `0.0.0.0`）
  - port：預設 `8080`
- middleware
  - normalize_path：NormalizePath（預設 true）
  - compress：Compress（預設 true）
  - logger
    - enabling：是否啟用請求日誌（預設 true）
    - format：支援 `%{url}xi`、`%s`、`%{Referer}i`、`%{User-Agent}i` 等佔位
- watch：是否啟用檔案監看（預設 true）
- templating
  - hot_reload：是否熱重載模板（預設 true）；亦允許 `Cofg::get(true)` 重新載入設定
  - value：`name:value` 陣列；型別支援 bool、i64、string；`name:env:ENV` 會讀 env
- toc
  - make_toc：是否生成 TOC（預設 true）
  - path：相對於 `public_path` 的輸出（預設 `index.html`）
  - ext：掃描副檔名清單（預設 `html,pdf,md`）
- public_path：內容根目錄（預設 `./public/`）

額外注入的模板變數：

- `server-version`（全域）
- `path`（每頁，為去除 `public_path` 前綴後的相對路徑）

## 運作細節

- 啟動流程：初始化 logger/目錄 → 清理舊 `.html`（僅清與 md 同名者）→ 啟動 HTTP 伺服（背景 thread）→ 檔案監看（可關閉）。
- 請求處理：
  - `GET /`：若存在 `public/index.html` 則回傳；否則即時渲染 TOC。
  - 其他路徑：
    - `.md`：讀檔並即時渲染成 HTML 回傳。
    - 其他副檔名：作為靜態檔回傳（找不到則回 `meta/404.html`）。
- 檔案監看：變更後收斂事件（1 秒 debounce），一次性執行：
  - `Cofg::get(true)`（若 `templating.hot_reload=true`）
  - `md2html_all()`（批次轉檔）
  - `make_toc()`（若 `toc.make_toc=true`）
- 日誌：URL 先 percent-decode 再寫入。

## Docker 使用

本倉庫含多階段 Dockerfile（Debian 基底）。建議做法：

1. 建置映像：

```pwsh
docker build -t my-http-server .
```

1. 準備容器設定檔（重點：綁定到 0.0.0.0）

```yaml
# cofg.yaml（容器建議值）
addrs:
  ip: 0.0.0.0
  port: 8080
watch: false # 依需求；容器通常可關閉監看
templating:
  hot_reload: false # 依需求；關閉可降噪
toc:
  make_toc: true
public_path: ./public/
```

1. 執行容器（掛載內容與設定檔）：

```pwsh
docker run --rm -p 8080:8080 `
  -v ${PWD}/public:/app/public `
  -v ${PWD}/cofg.yaml:/app/cofg.yaml `
  my-http-server
```

備註：若未掛載 `cofg.yaml` 且預設仍綁 `127.0.0.1`，容器對外將不可達。

### 使用 docker-compose（推薦）

本倉庫已提供可用範例：`docker-compose.yml` 與 `docker/cofg.docker.yaml`。

1. 確認 `public/` 內已有 Markdown 內容（或空資料夾也可）。

2. 啟動：

```pwsh
docker compose up -d --build
```

注意事項：

- 若你有掛載 `./meta:/app/meta`，請確保 `meta/` 內至少包含下列兩個檔案，否則會出現 500（模板缺失）：
  - `meta/html-t.templating`
  - `meta/404.html`
    如不想自備模板，可移除該 volume，改用容器內建的預設模板。
- `docker/cofg.docker.yaml` 內已示範容器建議配置（綁定 `0.0.0.0:8080`）。
- 可用環境變數注入模板變數。例如在 `docker-compose.yml` 設定：

  - `SITE_NAME=My Awesome Site`
    並在 `docker/cofg.docker.yaml` 加入：

  ```yaml
  templating:
    hot_reload: true
    value:
      - "site_name:env:SITE_NAME"
  ```

  之後你可以在模板中使用 `{{ SITE_NAME }}`。

1. 存取：

瀏覽器開啟 `http://localhost:8080/`。

環境變數示例：在 `docker-compose.yml` 中已示範 `SITE_NAME`，並在 `docker/cofg.docker.yaml` 透過 `templating.value` 的 `name:env:ENV_NAME` 注入模板。

## 測試

偏好使用 nextest（若未安裝可改 `cargo test`）：

```pwsh
cargo nextest run --no-fail-fast
```

測試涵蓋：設定載入/熱重載、模板 context、Markdown → HTML 基本渲染、TOC 生成等。測試檔位於 `src/test/`。

## CI / 發佈

- 安全稽核：`.github/workflows/Security-audit.yml` 例行執行 `rustsec/audit-check`。
- 建置/發佈：`.github/workflows/cli.yml` 交叉編譯並釋出 artifacts。

### GPG 簽署

發佈流程會自動對 release artifacts 進行 GPG 簽署。需要在 GitHub repository secrets 中設定：

- `GPG_PRIVATE_KEY`：GPG 私鑰（ASCII-armored 格式）
- `GPG_PASSPHRASE`：GPG 私鑰密碼

簽署檔案將以 `.sig` 副檔名附加至 GitHub release，可用於驗證下載檔案的完整性。

## 常見問題（FAQ）

- 啟動失敗：Port 已被占用？調整 `addrs.port`。
- 空白頁或 404：檢查 `meta/html-t.templating`、`meta/404.html` 與 `public/` 內容是否齊備。
- 變更未生效：若需熱重載請將 `templating.hot_reload=true`；否則需重啟。
- `.html` 被刪？只會刪除「與 Markdown 同名」的 `.html` 以避免舊輸出殘留。

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

歡迎開 Issue 與 PR。提交前請先執行測試以確保通過。
