# my-http-server

輕量級靜態 Markdown 轉 HTML 的 HTTP 伺服器，使用 Rust 與 actix-web。

主要功能：

- 自動把 public 目錄下的 Markdown (.md/.markdown) 轉換成 HTML 並提供靜態檔案伺服。
- 使用 actix_web 作為伺服器，並支援請求記錄中介層（Logger）。

## 安裝與建置

1. 取得原始碼：

   ```sh
   git clone https://github.com/Paul-16098/my-http-server.git
   cd my-http-server
   ```

2. 建置：

   cargo build --release

## 執行

將要公開的 Markdown 檔放到專案根目錄下的 public 資料夾（若沒有請建立），然後啟動：
`./my-http-server.exe`

預設行為會監聽本機的 127.0.0.1:8080。

## 範例

- 把 README.md 放進 `public/`，然後在瀏覽器開啟 `http://127.0.0.1:8080/README.html`（或依照程式路由）即可看到轉換後的 HTML。

## 環境變數

- REQUEST_LOGGER：`actix_web::middleware::logger::Logger` 使用的格式字串。預設為
  `%{url}xi %s "%{Referer}i" "%{User-Agent}i"`

## 設定檔

cofg.yaml:

```yaml
ip: 127.0.0.1
port: 8080
```

./meta/html-t.html: a html for base

./meta/404.html: 404 page

## 授權與貢獻

歡迎開 PR 與 issues。請於提交前確認測試通過。
