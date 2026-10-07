//! Checking what the user typed before any server is started.

use super::language::Language;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

pub fn validate(language: &dyn Language, symbol: &str, new_name: &str) -> Result<()> {
    let mut chars = new_name.chars();
    let starts_well = chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic());
    if !starts_well || !chars.all(|character| language.is_identifier_char(character)) {
        bail!(
            "`{new_name}` is not a plain {} identifier (letters, digits and underscores)",
            language.name()
        );
    }
    if language.reserved_words().contains(&new_name) {
        bail!(
            "`{new_name}` is a {} keyword and cannot name a symbol",
            language.name()
        );
    }
    if symbol == new_name {
        bail!("The new name equals the current name `{symbol}`");
    }
    if let Some(reason) = language.refuse_names(symbol, new_name) {
        bail!("{reason}");
    }
    Ok(())
}

/// The file to rename in: it exists, lies in the project and has one of the
/// language's extensions. Relative paths are taken from the project root.
pub fn resolve_file(language: &dyn Language, file: &Path, root: &Path) -> Result<PathBuf> {
    let absolute = if file.is_absolute() {
        file.to_path_buf()
    } else {
        root.join(file)
    };
    let file = absolute
        .canonicalize()
        .with_context(|| format!("File does not exist: {}", absolute.display()))?;
    if !file.starts_with(root) {
        bail!(
            "{} is outside the project {}",
            file.display(),
            root.display()
        );
    }
    let extension = file.extension().and_then(|extension| extension.to_str());
    if !extension.is_some_and(|extension| language.extensions().contains(&extension)) {
        bail!(
            "{} symbol rename needs a file ending in {}; got {}",
            language.name(),
            language
                .extensions()
                .iter()
                .map(|extension| format!(".{extension}"))
                .collect::<Vec<_>>()
                .join(" or "),
            file.display()
        );
    }
    Ok(file)
}

#[cfg(test)]
mod tests;
