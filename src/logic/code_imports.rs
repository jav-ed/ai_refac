//! Code that imports a page or a picture the Markdown driver is about to move.
//!
//! `import art from "../assets/hero.svg"` in a component and `import Card from
//! "../Card.astro"` in an `.mdx` page break when the file moves. The
//! TypeScript engine fixes them, planning from the files where they still are,
//! so it runs before the Markdown driver moves anything. It does not rename
//! those files: the Markdown driver does, and keeps fixing the Markdown links
//! exactly as it does for a request without code.

use super::report::Pairs;
use crate::drivers::RefactorDriver;
use crate::drivers::typescript::TypeScriptDriver;
use anyhow::{Context, Result};
use std::path::Path;

/// The groups of a request: language, its pairs, and its driver.
pub(super) type Groups = Vec<(String, Pairs, Box<dyn RefactorDriver>)>;

/// What the engine did.
pub(super) struct Fixed {
    /// The TypeScript pairs of the request, which the engine moved in the same
    /// pass; their group is taken out of `groups`.
    pub typescript_files: Pairs,
    pub notes: Vec<String>,
}

/// Fixes the code that imports the `companion` pairs, and moves the request's
/// own TypeScript. An error means nothing has moved or changed.
pub(super) async fn fix_before_moving(
    groups: &mut Groups,
    companion: &Pairs,
    root: Option<&Path>,
) -> Result<Fixed> {
    // What the Markdown driver would refuse, it refuses now: once the code is
    // fixed for the new places, a move that then fails would leave it pointing
    // at files that did not move.
    if let Some((_, files, driver)) = groups.iter().find(|(lang, ..)| lang == "markdown") {
        driver
            .plan_move(files.clone(), root)
            .await
            .context("The pages and pictures cannot be moved, so nothing was changed")?;
    }

    let typescript_files = groups
        .iter()
        .position(|(lang, ..)| lang == "typescript")
        .map(|index| groups.remove(index).1)
        .unwrap_or_default();
    let notes = TypeScriptDriver
        .move_alongside(typescript_files.clone(), companion.clone(), root)
        .await
        .context("Code that imports the files to move could not be fixed, so nothing was moved")?;
    Ok(Fixed {
        typescript_files,
        notes,
    })
}
