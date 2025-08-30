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

## 設定檔

./cofg.yaml: [src/cofg.yaml](https://github.com/Paul-16098/my-http-server/blob/main/src/cofg/cofg.yaml)

./meta/html-t.templating: a templating for base

./meta/404.html: 404 page

## tree

```tree
 src
 ┣ cofg -- cofg
 ┃ ┣ cofg.yaml -- cofg Default
 ┃ ┗ mod.rs -- cofg main
 ┣ parser -- parser
 ┃ ┣ markdown.rs -- parser md
 ┃ ┣ mod.rs -- parser main
 ┃ ┗ templating.rs -- parser main
 ┣ test -- test
 ┃ ┣ cofg.rs -- test for cofg
 ┃ ┗ mod.rs-- test main
 ┗ main.rs -- main
```

## 授權與貢獻

歡迎開 PR 與 issues。請於提交前確認測試通過。
