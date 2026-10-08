//! The `super::` paths of a module that moves: they are position dependent,
//! so the ones that leave the moved module are written out as `crate::` paths.

use super::apply::TextReplacement;
use anyhow::{Result, bail};
use ra_ap_ide::TextRange;
use ra_ap_syntax::{
    AstNode, Edition, SourceFile,
    ast::{self, HasName},
};
use std::path::Path;

/// Rewrites the `super::` paths of a moved file whose target lies outside the
/// moved module into absolute `crate::` paths, so they keep their meaning in the
/// new place. A `super` that stays inside the moved module (the `use super::*`
/// of an inline `mod tests`, a child reaching its parent) is position
/// independent and stays as written. `moved_module` is the old path of the
/// module being moved.
pub fn super_path_edits(
    path: &Path,
    content: &str,
    file_module: &[String],
    moved_module: &[String],
) -> Result<Vec<TextReplacement>> {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    if !parse.errors().is_empty() {
        bail!(
            "Cannot move {} because it contains Rust syntax errors",
            path.display()
        );
    }

    let mut edits = Vec::new();
    for candidate in parse
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Path::cast)
    {
        if candidate
            .syntax()
            .ancestors()
            .skip(1)
            .any(|ancestor| ast::Path::cast(ancestor).is_some())
        {
            continue;
        }

        // Only the leading `super::` segments are rewritten; whatever follows
        // them (generic arguments, `::` paths of any shape) stays as written.
        let segments: Vec<ast::PathSegment> = candidate.segments().collect();
        let super_count = segments
            .iter()
            .take_while(|segment| matches!(segment.kind(), Some(ast::PathSegmentKind::SuperKw)))
            .count();
        if super_count == 0 {
            continue;
        }

        let mut context = file_module.to_vec();
        let mut inline_modules = candidate
            .syntax()
            .ancestors()
            .skip(1)
            .filter_map(ast::Module::cast)
            .filter_map(|module| module.name().map(|name| name.text().to_string()))
            .collect::<Vec<_>>();
        inline_modules.reverse();
        context.extend(inline_modules);

        if super_count > context.len() {
            bail!(
                "Path `{}` in {} climbs above the crate root",
                candidate.syntax().text(),
                path.display()
            );
        }
        context.truncate(context.len() - super_count);
        if context.starts_with(moved_module) {
            continue;
        }
        let mut replacement = if context.is_empty() {
            "crate".to_string()
        } else {
            format!("crate::{}", context.join("::"))
        };
        // `pub(super)` becomes `pub(in crate::parent)`: a path in a visibility
        // needs the `in`, only the bare keywords go without it.
        let visibility_without_in = candidate
            .syntax()
            .parent()
            .and_then(ast::VisibilityInner::cast)
            .is_some_and(|inner| inner.in_token().is_none());
        if visibility_without_in {
            replacement = format!("in {replacement}");
        }
        let leading_supers = TextRange::new(
            candidate.syntax().text_range().start(),
            segments[super_count - 1].syntax().text_range().end(),
        );
        edits.push(TextReplacement::from_range(
            path.to_path_buf(),
            leading_supers,
            replacement,
        ));
    }
    Ok(edits)
}
