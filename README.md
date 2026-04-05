# my-http-server

高效能 Rust HTTP 伺服器，支援靜態檔案串流與 Markdown 動態渲染，搭配 Actix-web、markdown-ppp、Handlebars 模板與中介軟體（速率限制、Basic Auth、IP 過濾、壓縮、日誌）。

## 專案簡述與關鍵目標

- 即時將 `.md` 轉為 HTML，並注入可客製的 Handlebars 模板
- 靜態檔案零拷貝串流，預設目錄 `public/`
- 多層配置：預設值 → 檔案 → 環境變數 → CLI 參數
- 開發友善熱重載：模板/配置更新立即生效（dev）
- 生產可用中介軟體：速率限制、HTTP Basic Auth、IP 過濾、壓縮、日誌、路徑正規化

## 技術棧

| 元件 | 技術 | 版本 |
| --- | --- | --- |
| 語言 | Rust | Edition 2024 |
| Web | Actix-web | 4.11.0 |
| 靜態檔案 | Actix-files | 0.6.8 |
| 模板 | Handlebars | 6.3.2 |
| Markdown | markdown-ppp | 2.7.1 |
| TLS | rustls | 0.23 |
| 速率限制 | actix-governor | 0.10.0 |
| IP 過濾 | actix-ip-filter | 0.3.2 |
| CLI | clap | 4.5.49 |
| Glob | wax | 0.6.0 |
| OpenAPI（選用） | utoipa | 5.4.0 |
| HTTP Auth | actix-web-httpauth | 0.8.2 |

### 可選功能旗標

- `github_emojis`：抓取並快取 GitHub emoji
- `api`：啟用 /api OpenAPI/Swagger UI

## 架構與資料流

### 請求管線

```text
HTTP Request
  ↓ route: / 或 /{filename:.*}
  ↓ cached_public_req_path：解碼 URI、解析磁碟路徑
  ├─ 若副檔名 .md：讀檔 → md2html → Handlebars 模板 → 回應 HTML
  ├─ 若 public/index.html 存在：直接回傳（使用者優先）
  ├─ 若路徑為 /：生成 TOC Markdown → md2html → 回應
  └─ 其他：actix_files::NamedFile 串流（404 則回傳 404 頁）
```

### 模組職責

| 模組 | 職責 | 主要介面 |
| --- | --- | --- |
| `src/request.rs` | 路由處理、路徑解析、分支邏輯 | `index`、`/{filename}`、`cached_public_req_path()` |
| `src/main.rs` | 伺服器啟動、中介軟體鏈 | `main()`、`build_server()` |
| `src/parser/mod.rs` | md → HTML → 模板管線協調 | `md2html()` |
| `src/parser/templating.rs` | Handlebars 生命週期、上下文組裝 | `get_engine()`、`get_context()` |
| `src/parser/markdown.rs` | TOC 建立、Markdown 解析工具 | `get_toc()` |
| `src/cofg/config.rs` | 配置快取、XDG 路徑、分層載入 | `Cofg::new()`、`Cofg::get()` |
| `src/error.rs` | 統一錯誤型別與 HTTP 回應 | `AppError`、Responder 實作 |

### 全域快取模式

- **配置**：`OnceCell<RwLock<Cofg>>`，首讀載入，後續快取；`Cofg::get(true)` 在 `templating.hot_reload=true` 時可強制重載
- **模板引擎**：`OnceCell<RwLock<Handlebars>>`，一般模式單次建構；熱重載模式每次重建
- **每請求快取**：解碼後的 URI、解析後路徑、是否 Markdown 副檔名

### 配置優先序（低→高）

1. 內建預設（`include_str!` 嵌入）
2. 配置檔（`./cofg.yaml`、`--config-path`、或 XDG 位置）
3. 環境變數（前綴 `MYHTTP_*`）
4. CLI 參數

### 初始化流程

解析 CLI → 若有 --root-dir 先切換 CWD
→ Cofg::init_global() 載入分層配置
→ 公開路徑 canonicalize
→ init() 建立 XDG 目錄與預設檔

### Markdown 渲染流程

```rust
md2html(md, cfg, extra_vars)
  engine = get_engine(cfg)
  ctx = get_context(cfg)            // 內含 server-version + 動態變數
  for v in extra_vars: set_context_value(ctx, v)
  ast = parse_markdown(md)
  fragment = markdown_ppp::render_html(ast)
  ctx.body = fragment
  output = engine.render("html-t", ctx)  // 模板 meta/html-t.hbs
```

## 快速開始

### 需求

- Rust 1.70+（建議透過 rustup 安裝）
- Git
- 若需 TLS：憑證/私鑰（Let's Encrypt 或自簽）

### 安裝與啟動

1. 下載專案

   ```bash
   git clone https://github.com/Paul-16098/my-http-server.git
   cd my-http-server
   ```

2. 建置與啟動

   ```bash
   cargo run
   ```

   首次啟動會建立預設 `meta/html-t.hbs`、`meta/404.html`、`cofg.yaml`，並初始化 XDG 目錄（若啟用）。
3. 造訪服務
   - <http://localhost:8080>
   - `/`：若無 `index.html` 則回傳 TOC
   - `/path/to/file.md`：即時渲染為 HTML

### 配置範例（cofg.yaml 摘要）

```yaml
addrs:
  ip: 0.0.0.0
  port: 8080

tls:
  enable: false
  cert: path/to/cert.pem
  key: path/to/key.pem

public_root: ./public

templating:
  hot_reload: true
  hbs_path: meta/html-t.hbs

template_data:
  - name: value
  - counter: 42
  - enabled: true
  - token:env:TOKEN_VAR

middleware:
  logger: {}
  rate_limiter: {}
  auth: {}
  ip_filter: {}
  compression: {}
```

### 熱重載

- `templating.hot_reload: true`（開發模式）
- 模板檔變更：下一次請求即生效
- 配置重載：需呼叫 `Cofg::get(true)`（僅在 hot_reload=true 時允許）
- 生產環境建議關閉以確保穩定

## 專案結構

```tree
my-http-server/
├─ src/
│  ├─ main.rs              # 伺服器啟動與中介軟體鏈
│  ├─ request.rs           # 路由與路徑解析
│  ├─ error.rs             # AppError 與 Responder
│  ├─ parser/
│  │  ├─ mod.rs            # md2html 協調
│  │  ├─ templating.rs     # Handlebars 引擎與上下文
│  │  └─ markdown.rs       # TOC / Markdown 工具
│  ├─ cofg/
│  │  ├─ config.rs         # 配置結構與快取
│  │  ├─ cli.rs            # CLI 參數解析
│  │  ├─ cofg.yaml         # 預設配置
│  │  └─ mod.rs
│  ├─ api/                 # OpenAPI（feature: api）
│  └─ test/                # 單元/整合/安全測試
├─ meta/                   # 模板與 404 頁面
├─ public/                 # 靜態檔案根目錄
├─ architecture.md         # 架構與資料流
├─ Cargo.toml              # 依賴與中繼資料
├─ cofg.yaml               # 預設配置檔
├─ Makefile.toml           # build/test/coverage 任務
├─ CHANGELOG.md
└─ LICENSE.txt
```

## 主要特性

1) Markdown 即時渲染：AST 轉 HTML，注入模板，支援型別推斷與 env 變數
2) 靜態檔案零拷貝串流：`actix-files::NamedFile`
3) TOC 自動生成：無 `index.html` 時提供導覽
4) 彈性配置系統：四層優先序，支援 XDG，env/CLI 覆寫
5) 中介軟體管線：速率限制、Basic Auth、IP 過濾、壓縮、日誌、路徑正規化
6) 熱重載：模板與配置可開發期即時更新
7) TLS/HTTPS：rustls，無 OpenSSL 依賴
8) API 文件（選用）：`/api` Swagger UI（feature: api）
9) Emoji 支援（選用）：抓取並快取 GitHub emoji（feature: github_emojis）

## 開發流程

### 建置

```bash
cargo build               # 開發
cargo build --release     # 發佈
cargo build --all-features
```

### 測試

```bash
cargo make test           # 全測試
cargo test config         # 篩選測試
cargo make cov            # 產生 lcov.info
cargo make html-cov       # 產生 HTML coverage
```

#### 測試分佈(src/test/)

- `cli.rs`：CLI 解析
- `config.rs`：配置載入、分層、熱重載
- `parser.rs`：Markdown/模板/上下文
- `request.rs`：路由與路徑解析
- `security.rs`：路徑穿越、認證、IP 過濾
- `integration.rs`：HTTP 整合
- `error.rs`：錯誤對應

### 靜態分析

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### 分支與釋出

- 預設分支：`dev`
- 流程：feature/fix 分支 → PR → 合併 `dev` → 發 Tag 觸發 changelog/多平台建置/發布

### 熱編輯循環

```bash
# 端 1
cargo run
# 端 2
curl http://localhost:8080/path/to/file.md
# 編輯 meta/html-t.hbs 或 cofg.yaml；下一次請求即反映（hot_reload=true）
```

## 編碼規範（重點）

- 遵循 Rust API Guidelines；`cargo clippy -- -D warnings`、`cargo fmt`
- 註解僅說明「為什麼」（WHY），避免解釋「做什麼」（WHAT）
- 公開 API 加 doc comment；複雜/效能/安全邏輯加 `// WHY:`
- 錯誤處理：回傳 `AppResult<T>`；避免 `unwrap/expect`；必要時先記錄日誌
- 性能：配置/模板快取；路徑解碼快取；靜態檔案串流；中介軟體順序優先早拒絕
- 安全：
  - 路徑：一律使用 `cached_public_req_path()`
  - 認證：使用常數時間比較 `constant_time_eq`、`ct_eq_str_opt`
  - 模板：配置值信任；若引入使用者輸入需先消毒
  - HTTPS：生產建議啟用 TLS
- 併發：避免在 `.await` 期間持有 RwLock 讀鎖；複製快取值再使用；寫鎖僅熱重載路徑

## 性能與安全注意

- 配置一次載入，多次複製；hot_reload=true 時才允許強制重載
- 模板引擎一般模式單次建構；熱重載每請求重建
- Markdown 每請求新解析（如需可加快取）
- 靜態檔案使用 zero-copy；中介軟體順序：rate limit → log → normalize → compress → auth → IP filter
- 安全：防路徑穿越、常數時間比較、TLS 建議啟用

## 故障排解

| 問題 | 解法 |
| --- | --- |
| 模板未更新 | 確認 `templating.hot_reload: true`，並重新請求 |
| Markdown 404 | 檢查檔案是否在 `public/`，或調整 `public_root` |
| 認證失敗 | 確認已設定環境變數/配置的帳密與啟用旗標 |
| 渲染變慢 | 檢查檔案大小、可用 `cargo flamegraph` 分析 |
| 埠被占用 | 修改 `addrs.port` 或釋放占用行程 |

## 資源

- 架構：`architecture.md`
- 配置模式：`src/cofg/cofg.yaml`
- CLI：`my-http-server --help`
- 任務：`Makefile.toml`
- GitHub：<https://github.com/Paul-16098/my-http-server>

## 版本資訊

- 版本：4.1.5
- 更新日期：2026-01-28
