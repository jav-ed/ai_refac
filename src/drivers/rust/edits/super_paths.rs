//! The `super::` paths of a module that moves: they are position dependent,
//! so the ones that leave the moved module are written out as `crate::` paths.

use super::macro_paths::{outermost_token_trees, pieces_of};
use crate::drivers::rust::transaction::apply::TextReplacement;
use anyhow::{Result, bail};
use ra_ap_ide::TextRange;
use ra_ap_syntax::{
    AstNode, Edition, SourceFile, SyntaxNode,
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

        let mut context = module_of(candidate.syntax(), file_module);
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
        let mut replacement = absolute_path(&context);
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

/// The logical module a node sits in: the module of its file plus the inline
/// `mod name { … }` blocks around it.
fn module_of(node: &SyntaxNode, file_module: &[String]) -> Vec<String> {
    let mut context = file_module.to_vec();
    let mut inline_modules = node
        .ancestors()
        .skip(1)
        .filter_map(ast::Module::cast)
        .filter_map(|module| module.name().map(|name| name.text().to_string()))
        .collect::<Vec<_>>();
    inline_modules.reverse();
    context.extend(inline_modules);
    context
}

fn absolute_path(module: &[String]) -> String {
    if module.is_empty() {
        "crate".to_string()
    } else {
        format!("crate::{}", module.join("::"))
    }
}

/// The same rewrite for `super::` paths in the arguments of macro calls
/// (`assert_eq!(super::helper(), 1)`), which are tokens and not path nodes.
/// The body of a `macro_rules!` definition is left alone: its `super` means the
/// module of every call site, not of the definition.
pub fn super_macro_path_edits(
    path: &Path,
    content: &str,
    file_module: &[String],
    moved_module: &[String],
) -> Vec<TextReplacement> {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    let mut edits = Vec::new();
    for tree in outermost_token_trees(&parse.tree()) {
        if tree.syntax().ancestors().any(|node| {
            ast::MacroRules::can_cast(node.kind()) || ast::MacroDef::can_cast(node.kind())
        }) {
            continue;
        }
        let pieces = pieces_of(&tree);
        let mut index = 0;
        while index < pieces.len() {
            if pieces[index].text != "super" || (index > 0 && pieces[index - 1].text == "::") {
                index += 1;
                continue;
            }
            let mut last = index;
            let mut count = 1;
            while pieces.get(last + 1).is_some_and(|piece| piece.text == "::")
                && pieces
                    .get(last + 2)
                    .is_some_and(|piece| piece.text == "super")
            {
                last += 2;
                count += 1;
            }
            // A `super` that no `::` follows is not the start of a path.
            let is_path = pieces.get(last + 1).is_some_and(|piece| piece.text == "::");
            let mut context = module_of(tree.syntax(), file_module);
            if is_path && count <= context.len() {
                context.truncate(context.len() - count);
                if !context.starts_with(moved_module) {
                    edits.push(TextReplacement::from_range(
                        path.to_path_buf(),
                        pieces[index].range.cover(pieces[last].range),
                        absolute_path(&context),
                    ));
                }
            }
            index = last + 1;
        }
    }
    edits
}

#[cfg(test)]
mod tests;
