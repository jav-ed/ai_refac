use crate::drivers::kotlin::rename::rename_symbol as rename_kotlin_symbol;
use crate::drivers::lsp_rename::languages::{Dart, Go, Python, Rust};
use crate::drivers::lsp_rename::rename_symbol as rename_with;
use crate::drivers::symbol_rename::{RenameReport, RenameRequest};
use crate::drivers::typescript::rename::rename_symbol as rename_typescript_symbol;
use anyhow::{Result, bail};

/// Central entry point for symbol renames: picks the language backend from the
/// file extension. TypeScript/JavaScript and Kotlin have their own backends;
/// the other languages share the language-server engine.
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
        "go" => rename_with(&Go, request).await,
        "rs" => rename_with(&Rust, request).await,
        "py" => rename_with(&Python, request).await,
        "dart" => rename_with(&Dart, request).await,
        _ => bail!(
            "Symbol rename supports TypeScript/JavaScript (.ts .tsx .js .jsx .mts .cts .mjs .cjs), Kotlin (.kt), Go (.go), Rust (.rs), Python (.py) and Dart (.dart) files; got {}",
            request.file.display()
        ),
    }
}
