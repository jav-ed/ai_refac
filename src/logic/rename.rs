use crate::drivers::kotlin::rename::{
    rename_all_symbols as rename_all_kotlin, rename_symbol as rename_kotlin_symbol,
};
use crate::drivers::lsp::rename::languages::{Dart, Go, Python, Rust};
use crate::drivers::lsp::rename::{
    rename_symbol as rename_with, rename_symbols as rename_all_with,
};
use crate::drivers::symbol::rename::{RenameReport, RenameRequest};
use crate::drivers::typescript::rename::rename_symbol as rename_typescript_symbol;
use anyhow::{Result, bail};
use std::path::Path;

/// The backend that renames symbols of a file, picked from its extension.
/// TypeScript/JavaScript and Kotlin have their own backends; the other
/// languages share the language-server engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    TypeScript,
    Kotlin,
    Go,
    Rust,
    Python,
    Dart,
}

impl Backend {
    fn of(file: &Path) -> Result<Self> {
        let extension = file
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("");
        match extension {
            "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs" => Ok(Self::TypeScript),
            "kt" => Ok(Self::Kotlin),
            "go" => Ok(Self::Go),
            "rs" => Ok(Self::Rust),
            "py" => Ok(Self::Python),
            "dart" => Ok(Self::Dart),
            _ => bail!(
                "Symbol rename supports TypeScript/JavaScript (.ts .tsx .js .jsx .mts .cts .mjs .cjs), Kotlin (.kt), Go (.go), Rust (.rs), Python (.py) and Dart (.dart) files; got {}",
                file.display()
            ),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::TypeScript => "TypeScript/JavaScript",
            Self::Kotlin => "Kotlin",
            Self::Go => "Go",
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Dart => "Dart",
        }
    }
}

/// Central entry point for symbol renames: picks the language backend from the
/// file extension.
pub async fn handle_rename(request: RenameRequest) -> Result<RenameReport> {
    match Backend::of(&request.file)? {
        Backend::TypeScript => rename_typescript_symbol(request).await,
        Backend::Kotlin => rename_kotlin_symbol(request).await,
        Backend::Go => rename_with(&Go, request).await,
        Backend::Rust => rename_with(&Rust, request).await,
        Backend::Python => rename_with(&Python, request).await,
        Backend::Dart => rename_with(&Dart, request).await,
    }
}

/// Several renames of one project in one language-server session, all or
/// nothing: the server is started once, each rename is planned, proven and
/// written against the files as the one before left them, and a failure takes
/// the earlier renames back. The reports come back in the order of the
/// requests.
pub async fn handle_rename_batch(requests: Vec<RenameRequest>) -> Result<Vec<RenameReport>> {
    let Some(first) = requests.first() else {
        bail!("A batch needs at least one rename");
    };
    let backend = Backend::of(&first.file)?;
    for (index, request) in requests.iter().enumerate().skip(1) {
        let other = Backend::of(&request.file)?;
        if other != backend {
            bail!(
                "A batch renames in one language, but rename {} is a {} file ({}) and rename 1 a {} file ({}). Run one batch per language.",
                index + 1,
                other.name(),
                request.file.display(),
                backend.name(),
                first.file.display()
            );
        }
    }
    match backend {
        Backend::TypeScript => bail!(
            "A batch is available for Kotlin, Go, Rust, Python and Dart. The TypeScript/JavaScript rename has its own engine and takes one rename per command."
        ),
        Backend::Kotlin => rename_all_kotlin(requests).await,
        Backend::Go => rename_all_with(&Go, requests).await,
        Backend::Rust => rename_all_with(&Rust, requests).await,
        Backend::Python => rename_all_with(&Python, requests).await,
        Backend::Dart => rename_all_with(&Dart, requests).await,
    }
}

#[cfg(test)]
mod tests;
