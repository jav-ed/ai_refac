//! A module that moves out of the prefix of a grouped import. In
//! `use super::{Located, names, report};` the item `names` cannot be written
//! as a path below `super` once the module lives somewhere else, so it leaves
//! the group and gets a `use` line of its own right after it.

use crate::drivers::rust::transaction::apply::TextReplacement;
use anyhow::{Result, bail};
use ra_ap_syntax::{
    AstNode,
    ast::{self, HasAttrs, HasVisibility},
};
use std::path::Path;

/// The two edits that take the leaf `leaf` out of its group and import
/// `new_path` in a line of its own, or `None` when the leaf is not an item of
/// a group. The rest of the item (alias, glob, nested list) goes along.
/// `content` is the file the leaf is in.
pub fn split_leaf_from_group(
    file: &Path,
    content: &str,
    leaf: &ast::Path,
    new_path: &str,
) -> Result<Option<Vec<TextReplacement>>> {
    let Some(tree) = leaf.syntax().parent().and_then(ast::UseTree::cast) else {
        return Ok(None);
    };
    let Some(list) = tree.syntax().parent().and_then(ast::UseTreeList::cast) else {
        return Ok(None);
    };
    let Some(item) = list.syntax().ancestors().find_map(ast::Use::cast) else {
        return Ok(None);
    };
    if item.attrs().next().is_some() {
        bail!(
            "The import in {} carries attributes, so a module in its group cannot be moved out of it; split the group and retry",
            file.display()
        );
    }
    let siblings: Vec<ast::UseTree> = list.use_trees().collect();

    // What follows the path in the tree travels with it: an alias, a glob, or
    // a list of its own (`module_graph::{self, ResolvedModule}`).
    let tail = &content[usize::from(leaf.syntax().text_range().end())
        ..usize::from(tree.syntax().text_range().end())];
    let visibility = item
        .visibility()
        .map(|visibility| format!("{} ", visibility.syntax().text()))
        .unwrap_or_default();
    let item_range = item.syntax().text_range();
    let edit = |start: usize, end: usize, replacement: String| TextReplacement {
        path: file.to_path_buf(),
        start,
        end,
        replacement,
    };

    // A group of one: the whole import is the moved module, written afresh.
    if siblings.len() == 1 {
        let prefix_is_the_root = list
            .syntax()
            .parent()
            .and_then(ast::UseTree::cast)
            .is_some_and(|prefix| prefix.syntax().parent() == Some(item.syntax().clone()));
        if !prefix_is_the_root {
            return Ok(None);
        }
        return Ok(Some(vec![edit(
            usize::from(item_range.start()),
            usize::from(item_range.end()),
            format!("{visibility}use {new_path}{tail};"),
        )]));
    }

    // Take the item out of the group. Of a pair, the other one is left alone
    // and loses its braces; otherwise the item goes with one comma: the one
    // after it, or the one before it when it is the last of the group.
    let position = siblings
        .iter()
        .position(|candidate| candidate.syntax() == tree.syntax())
        .expect("a tree of a list is one of its siblings");
    let removal = if siblings.len() == 2 {
        let other = &siblings[1 - position];
        edit(
            usize::from(list.syntax().text_range().start()),
            usize::from(list.syntax().text_range().end()),
            other.syntax().text().to_string(),
        )
    } else {
        match siblings.get(position + 1) {
            Some(next) => edit(
                usize::from(tree.syntax().text_range().start()),
                usize::from(next.syntax().text_range().start()),
                String::new(),
            ),
            None => edit(
                usize::from(siblings[position - 1].syntax().text_range().end()),
                usize::from(tree.syntax().text_range().end()),
                String::new(),
            ),
        }
    };

    let item_start = usize::from(item_range.start());
    let indent: String = content[..item_start]
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .take_while(|character| character.is_whitespace())
        .collect();
    let after_item = usize::from(item_range.end());
    Ok(Some(vec![
        removal,
        edit(
            after_item,
            after_item,
            format!("\n{indent}{visibility}use {new_path}{tail};"),
        ),
    ]))
}

#[cfg(test)]
mod tests;
