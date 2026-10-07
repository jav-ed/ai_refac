use crate::drivers::typescript::rename::{RenameReport, RenameRequest, rename_symbol};
use anyhow::{Result, bail};

/// Central entry point for symbol renames: picks the language backend from the
/// file extension. TypeScript/JavaScript is the first language with a rename.
pub async fn handle_rename(request: RenameRequest) -> Result<RenameReport> {
    let extension = request
        .file
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    match extension {
        "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs" => rename_symbol(request).await,
        _ => bail!(
            "Symbol rename supports TypeScript/JavaScript files only (.ts .tsx .js .jsx .mts .cts .mjs .cjs); got {}",
            request.file.display()
        ),
    }
}
