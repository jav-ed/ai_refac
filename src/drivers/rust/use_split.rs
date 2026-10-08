//! A module that moves out of the prefix of a grouped import. In
//! `use super::{Located, names, report};` the item `names` cannot be written
//! as a path below `super` once the module lives somewhere else, so it leaves
//! the group and gets a `use` line of its own right after it.

use super::apply::TextReplacement;
use anyhow::{Result, bail};
use ra_ap_syntax::{
    AstNode,
    ast::{self, HasAttrs, HasVisibility},
};
use std::path::Path;

/// The two edits that take the leaf `leaf` out of its group and import
/// `new_path` in a line of its own, or `None` when the leaf is not an item of
/// a group. `content` is the file the leaf is in.
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
    if tree.use_tree_list().is_some() || tree.star_token().is_some() {
        return Ok(None);
    }
    if item.attrs().next().is_some() {
        bail!(
            "The import in {} carries attributes, so a module in its group cannot be moved out of it; split the group and retry",
            file.display()
        );
    }
    let siblings: Vec<ast::UseTree> = list.use_trees().collect();
    if siblings.len() < 2 {
        return Ok(None);
    }

    // Take the item out together with one comma: the one after it, or the one
    // before it when it is the last of the group.
    let position = siblings
        .iter()
        .position(|candidate| candidate.syntax() == tree.syntax())
        .expect("a tree of a list is one of its siblings");
    let (start, end) = match siblings.get(position + 1) {
        Some(next) => (
            usize::from(tree.syntax().text_range().start()),
            usize::from(next.syntax().text_range().start()),
        ),
        None => (
            usize::from(siblings[position - 1].syntax().text_range().end()),
            usize::from(tree.syntax().text_range().end()),
        ),
    };

    let alias = tree
        .rename()
        .map(|rename| format!(" {}", rename.syntax().text()))
        .unwrap_or_default();
    let visibility = item
        .visibility()
        .map(|visibility| format!("{} ", visibility.syntax().text()))
        .unwrap_or_default();
    let item_start = usize::from(item.syntax().text_range().start());
    let indent: String = content[..item_start]
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .take_while(|character| character.is_whitespace())
        .collect();
    let after_item = usize::from(item.syntax().text_range().end());

    Ok(Some(vec![
        TextReplacement {
            path: file.to_path_buf(),
            start,
            end,
            replacement: String::new(),
        },
        TextReplacement {
            path: file.to_path_buf(),
            start: after_item,
            end: after_item,
            replacement: format!("\n{indent}{visibility}use {new_path}{alias};"),
        },
    ]))
}

#[cfg(test)]
mod tests;
