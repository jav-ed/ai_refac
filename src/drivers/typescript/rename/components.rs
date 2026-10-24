//! Component files a symbol rename would leave behind.
//!
//! The TypeScript 7 language server reads TypeScript and JavaScript. It does
//! not read inside `.astro` or `.mdx` files, so a name those files import or
//! use would stay the old one while the build goes on to fail, or worse, to
//! pass. The rename is refused with the files named, before anything is
//! written.

use super::plan::Plan;
use crate::drivers::typescript::{component_users, files};
use anyhow::{Result, bail};
use std::path::Path;

/// What the language server cannot read: `.astro` files too, unless `--astro`
/// has it read them through the content mapper.
fn unread_extensions(astro: bool) -> &'static [&'static str] {
    if astro { &["mdx"] } else { &["astro", "mdx"] }
}
/// Files listed in the error before it says how many more there are.
const LISTED: usize = 20;

pub async fn refuse_unread_users(
    project: &Path,
    symbol: &str,
    plan: &Plan,
    astro: bool,
) -> Result<()> {
    let extensions = unread_extensions(astro);
    if files::find(project, extensions)?.is_empty() {
        return Ok(());
    }
    let defining: Vec<_> = plan.files.iter().map(|file| file.path.clone()).collect();
    let users = component_users(project, symbol, &defining, extensions).await?;
    if users.is_empty() {
        return Ok(());
    }

    let shown: Vec<String> = users
        .iter()
        .take(LISTED)
        .map(|file| {
            file.strip_prefix(project)
                .unwrap_or(file)
                .display()
                .to_string()
        })
        .collect();
    let more = match users.len().saturating_sub(LISTED) {
        0 => String::new(),
        rest => format!("\n... and {rest} more"),
    };
    bail!(
        "`{symbol}` is imported by {} component file(s) that the TypeScript language server cannot read:\n{}{more}\n\
         Renaming it would leave those files on the old name. No files were changed. \
         Rename it in the TypeScript files and in these files by hand.",
        users.len(),
        shown.join("\n")
    );
}
