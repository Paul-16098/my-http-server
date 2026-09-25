//! parser
//!
//! WHY: Provides single entry point for Markdown → HTML → template flow.
//! Hot reload & context assembly handled in internal modules. (Global HTML caching removed
//! for simplicity: each request now re-renders Markdown.)

#[cfg(feature = "github_emojis")]
use std::collections::HashMap;
#[cfg(feature = "github_emojis")]
use std::sync::OnceLock;

use crate::parser::templating::set_context_value;

pub(crate) mod markdown;
pub(crate) mod templating;

#[cfg(feature = "github_emojis")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Emojis {
	pub(crate) unicode: HashMap<String, String>,
	pub(crate) r#else: HashMap<String, String>,
}
#[cfg(feature = "github_emojis")]
pub(crate) static EMOJIS: OnceLock<Emojis> = OnceLock::new();

/// Convert a single markdown string into full HTML page via template `html-t` (file: configured via `hbs_path`).
///
/// Steps:
/// 1. Acquire (or rebuild) template engine
/// 2. Build fresh context (server + configured vars)
/// 3. Apply extra template_data_list entries (e.g. `path:...`)
/// 4. Parse markdown → AST → HTML body
/// 5. Inject `body` then render compiled template
///
/// WHY: Keep side effects (engine caching, context assembly) localized while exposing a pure-ish
/// interface to callers. Accepts owned `md` so upstream can cheaply `read_to_string` and transfer
/// ownership without clone.
///
/// Contract / 契約（重要行為與邊界）
/// - Inputs:
///   - `md`: 原始 Markdown（UTF-8）。不做串流；一次性解析。
///   - `c`: 讀取模板設定與 hot reload 旗標；實際模板引擎取得見 `templating::get_engine`。
///   - `template_data_list`: 以 `name:value`（含 `name:env:ENV`）格式注入 Context；
///     智慧型別推斷順序為 bool → i64 → string；無冒號或格式不正者將被忽略。
///     後加入的條目會覆寫同名既有值（包含由設定檔注入者）。
/// - Output: 以邏輯名 `html-t` 渲染完成的完整 HTML 字串；Context 會包含：
///   - 由 `get_context` 注入的內建鍵：`server-version`
///   - 本函式注入的 `body`：Markdown 轉出的 HTML 片段
/// - Errors: 可能來自
///   - Markdown 解析失敗（語法錯誤或非預期情形）
///   - 模板檔案註冊/解析失敗（檔案缺失或模板語法錯誤）
///   - 模板渲染失敗（缺鍵/型別不符等）→ 包裝為 `AppError::RenderError`
/// - Side effects:
///   - 首次渲染若引擎尚未註冊 `html-t`，會以 `hbs_path` 進行註冊（讀檔）。
///   - 以 `trace` 層級輸出 AST（大型文件可能產生大量日誌）。
/// - Perf/Security notes:
///   - 正常模式引擎為快取重用；`hot_reload=true` 時每請求重建以反映模板改動。
///   - 渲染依賴本機模板檔案路徑；如內容根不可信，請配合上游路徑檢查避免 traversal。
pub(crate) fn md2html(
	md: String,
	c: &crate::cofg::config::Cofg,
	template_data_list: Vec<String>,
) -> crate::error::AppResult<String> {
	let mut engine = templating::get_engine(c)?;
	let mut context = templating::get_context(c);
	// NOTE: 後寫優先（呼叫端提供者可覆寫設定注入的鍵）。
	for template_data in template_data_list {
		set_context_value(&mut context, &template_data);
	}
	// Lazy 註冊模板：避免在未使用時就讀檔；同時配合 hot reload（引擎重建後將再次註冊）。
	// 使用 resolve_hbs_path 以支持 XDG 配置目錄優先級
	if !engine.has_template("html-t") {
		let hbs_path = c.resolve_hbs_path();
		engine.register_template_file("html-t", &hbs_path)?;
	}

	#[cfg(feature = "github_emojis")]
	let mut md = md;
	// Private Use Area Unicode characters used as temporary sentinels
	const SENTINEL_START: &str = "\u{E000}";
	const SENTINEL_END: &str = "\u{E001}";

	#[cfg(feature = "github_emojis")]
	/// 1. Pre-process raw markdown string: Protect escaped emojis (\:emoji:)
	pub fn protect_escaped_emojis(raw_md: &str, emojis: &Emojis) -> String {
		let mut text = raw_md.to_string();

		for k in emojis.unicode.keys() {
			let pat = format!(":{k}:");
			if !text.contains(&pat) {
				continue;
			}

			let mut result = String::with_capacity(text.len());
			let mut last_end = 0;

			for (start, _) in text.match_indices(&pat) {
				// Count backslashes immediately preceding :emoji:
				let backslash_count = text[..start]
					.chars()
					.rev()
					.take_while(|&c| c == '\\')
					.count();

				if backslash_count % 2 != 0 {
					// ODD backslashes: Escaped pattern (e.g. \:arrow_down:)
					// Protect the inner pattern so the AST Transformer skips it
					result.push_str(&text[last_end..start]);
					result.push(':');
					result.push_str(SENTINEL_START);
					result.push_str(k);
					result.push_str(SENTINEL_END);
					result.push(':');
				} else {
					// EVEN backslashes: Active pattern, keep as-is
					result.push_str(&text[last_end..start + pat.len()]);
				}
				last_end = start + pat.len();
			}
			result.push_str(&text[last_end..]);
			text = result;
		}
		text
	}

	/// Helper to strip sentinels
	fn restore_sentinels(text: &str) -> String {
		if text.contains(SENTINEL_START) {
			text.replace(SENTINEL_START, "").replace(SENTINEL_END, "")
		} else {
			text.to_string()
		}
	}

	#[cfg(feature = "github_emojis")]
	/// 2. AST Transformer
	struct ReplaceGithubEmojis<'a>(&'a Emojis);

	#[cfg(feature = "github_emojis")]
	impl<'a> markdown_ppp::ast_transform::Transformer for ReplaceGithubEmojis<'a> {
		fn transform_inline(
			&mut self,
			inline: markdown_ppp::ast::Inline,
		) -> markdown_ppp::ast::Inline {
			let e = self.0;
			match inline {
				// Only process plain text nodes
				markdown_ppp::ast::Inline::Text(mut text) => {
					for (k, v) in e.unicode.iter() {
						let pat = format!(":{k}:");
						if text.contains(&pat) {
							text = text.replace(&pat, v);
						}
					}
					// Restore protected escaped patterns back to literal text
					markdown_ppp::ast::Inline::Text(restore_sentinels(&text))
				}
				// Code blocks remain untouched, but clean up sentinels if any were inside code
				markdown_ppp::ast::Inline::Code(code) => {
					markdown_ppp::ast::Inline::Code(restore_sentinels(&code))
				}
				other => self.walk_transform_inline(other),
			}
		}
	}

	#[cfg(feature = "github_emojis")]
	// 1. Protect escaped emojis on the raw string FIRST
	{
		md = protect_escaped_emojis(&md, EMOJIS.get().unwrap());
	}
	// 2. Parse Markdown to AST
	let mut ast = markdown::parser_md(md)?;

	// 3. Run AST Transformer
	#[cfg(feature = "github_emojis")]
	{
		if let Some(emojis) = EMOJIS.get() {
			ast = markdown_ppp::ast_transform::Transform::transform_with(
				ast,
				ReplaceGithubEmojis(emojis),
			);
		}
	}
	// log::trace!("ast={:#?}", ast);
	let html = markdown_ppp::html_printer::render_html(
		&ast,
		markdown_ppp::html_printer::config::Config::default(),
	);

	// Contract: 模板預期取得 `body` 作為主要內容插槽。
	context.data_mut()["body"] = handlebars::JsonValue::String(html);
	match engine.render_with_context("html-t", &context) {
		Ok(o) => Ok(o),
		Err(o) => {
			log::error!("md2html:{}", o);
			Err(crate::error::AppError::RenderError(o))
		}
	}
}
