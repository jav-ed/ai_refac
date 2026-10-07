use crate::drivers::kotlin::rename::rename_symbol as rename_kotlin_symbol;
use crate::drivers::symbol_rename::{RenameReport, RenameRequest};
use crate::drivers::typescript::rename::rename_symbol as rename_typescript_symbol;
use anyhow::{Result, bail};

/// Central entry point for symbol renames: picks the language backend from the
/// file extension. TypeScript/JavaScript and Kotlin have a semantic rename.
pub async fn handle_rename(request: RenameRequest) -> Result<RenameReport> {
    let extension = request
        .file
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    match extension {
        "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs" => {
            rename_typescript_symbol(request).await
        }
        "kt" => rename_kotlin_symbol(request).await,
        _ => bail!(
            "Symbol rename supports TypeScript/JavaScript (.ts .tsx .js .jsx .mts .cts .mjs .cjs) and Kotlin (.kt) files only; got {}",
            request.file.display()
        ),
    }
}
