//! Template engine helpers
//!
//! WHY: Isolate template engine initialization & context population so markdown → HTML pipeline
//! stays pure/minimal (`md2html`). Engine instance is cached via `OnceCell<RwLock<_>>` for reuse
//! (bytecode cache active) unless `hot_reload` signals we must reconstruct to pick up file edits.

use handlebars::{Context, Handlebars};

use serde_json::json;
use std::sync::{OnceLock, RwLock};

use crate::error::{AppError, AppResult};

/// Inject a single `name:value` pair (auto type inference) into template context.
///
/// Supported forms:
/// - `foo:true/false` → bool
/// - `foo:123` → number (i64)
/// - `foo:some string` → string
/// - `foo:env:ENV_NAME` → reads `ENV_NAME` then infers (bool/number/string)
///
/// WHY: Allow configuration-driven variable list (`templating.value`) without schema explosion.
/// Parsing kept intentionally small (no floats) for predictability.
pub(crate) fn set_context_value(context: &mut Context, data: &str) {
    let context = context.data_mut();

    // Split only at the first ':'; skip malformed entries
    if let Some((name_raw, value_raw)) = data.split_once(':') {
        let name = name_raw.trim();
        let mut value = value_raw.trim().to_string();
        if name.is_empty() {
            return;
        }

        if value.starts_with("env:")
            && let Some((_, env)) = value.split_once(":")
        {
            if let Ok(v) = std::env::var(env) {
                value = v;
            } else {
                // Env var doesn't exist - skip this key entirely
                return;
            }
        }

        if let Ok(tf) = value.parse::<bool>() {
            context[name] = handlebars::JsonValue::Bool(tf);
            return;
        }

        // Parse numbers (i64 only)
        if let Ok(num) = value.parse::<i64>() {
            context[name] = handlebars::JsonValue::Number(num.into());
        } else {
            context[name] = handlebars::JsonValue::String(value);
        }
    } // else: no ':', ignore entry safely
}

/// Build a fresh template context with server metadata and configured variables.
///
/// Always includes `server-version`. Then folds `templating.value` items through
/// `set_context_value` to infer types.
///
/// WHY: Decouple config parsing from render path; context creation is cheap and explicit instead
/// of sharing mutable state across renders.
pub(crate) fn get_context(c: &crate::cofg::config::Cofg) -> Context {
    let mut context = Context::wraps(json!({
      "server-version": env!("CARGO_PKG_VERSION")
    }))
    .unwrap();
    if let Some(raw_str) = &c.templating.value {
        for data in raw_str {
            set_context_value(&mut context, data);
        }
    }

    context
}
static ENGINE: OnceLock<RwLock<Handlebars>> = OnceLock::new();

/// Retrieve (or rebuild under hot reload) the template engine.
///
/// On first call, initializes with bytecode cache (and optional hot reload). If `hot_reload` is
/// true, each call reconstructs a new engine to ensure latest template file contents are used.
///
/// WHY: Development ergonomics—trade small rebuild cost for immediacy when editing templates.
/// Production (no hot_reload) benefits from stable cached engine with bytecode reuse.
pub(crate) fn get_engine(c: &'_ crate::cofg::config::Cofg) -> AppResult<Handlebars<'_>> {
    let cell = ENGINE.get_or_init(|| {
        let mut e = Handlebars::new();
        if c.templating.hot_reload {
            e.set_dev_mode(true);
        }
        RwLock::new(e)
    });

    cell.read()
        .map(|e| e.clone())
        .map_err(|e| AppError::OtherError(e.to_string()))
}
