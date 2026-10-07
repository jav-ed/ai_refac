use crate::drivers::lsp_session::{LspSession, SessionConfig};
use anyhow::{Result, bail};
use serde_json::json;
use std::path::Path;

pub use crate::drivers::lsp_session::{LspSession as Session, RpcError};

/// Start the TypeScript 7 native engine on a project. rootUri is mandatory for
/// this server; UTF-8 positions keep every offset a plain byte offset in the
/// Rust-side text.
pub async fn start(executable: &Path, root: &Path) -> Result<Session> {
    let (mut session, init) = LspSession::start(SessionConfig {
        name: "TypeScript native engine",
        executable,
        args: vec!["--lsp".to_string(), "--stdio".to_string()],
        cwd: root,
        root,
        capabilities: json!({
            "general": { "positionEncodings": ["utf-8"] },
            "textDocument": { "rename": { "prepareSupport": true } },
        }),
        keep_notifications: &[],
        language_id,
    })
    .await?;
    let capabilities = &init["capabilities"];
    if capabilities["positionEncoding"] != "utf-8" {
        bail!(
            "The TypeScript engine does not support UTF-8 positions: {}",
            capabilities["positionEncoding"]
        );
    }
    if capabilities["renameProvider"].is_null() || capabilities["referencesProvider"].is_null() {
        bail!("The TypeScript engine does not offer rename and references");
    }
    session.initialized().await?;
    Ok(session)
}

fn language_id(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("tsx") => "typescriptreact",
        Some("jsx") => "javascriptreact",
        Some("js" | "mjs" | "cjs") => "javascript",
        _ => "typescript",
    }
}
