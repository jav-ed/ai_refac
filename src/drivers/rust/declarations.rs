//! Where a new `mod name;` line goes in the file of its parent. Appending at
//! the end of the file would leave it after the items and the tests module;
//! the declarations of a parent form a block at its top, usually in
//! alphabetical order, and the new one joins that block.

use super::apply::{TextReplacement, append_replacement};
use ra_ap_syntax::{
    AstNode, Edition, SourceFile,
    ast::{self, HasModuleItem, HasName},
};
use std::path::Path;

/// The edit that adds `declaration` (the text of a `mod <name>;` item, with
/// any attributes) to the file `content`. It goes before the first existing
/// declaration that sorts after `name`, or after the last one when the block
/// is not sorted or `name` sorts last. A file without declarations gets it
/// at the end.
pub fn insert_module_declaration(
    path: &Path,
    content: &str,
    name: &str,
    declaration: &str,
) -> TextReplacement {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    let declared: Vec<(String, usize, usize)> = parse
        .tree()
        .items()
        .filter_map(|item| match item {
            ast::Item::Module(module) if module.item_list().is_none() => {
                let range = module.syntax().text_range();
                Some((
                    module.name()?.text().to_string(),
                    line_start(content, usize::from(range.start())),
                    line_end(content, usize::from(range.end())),
                ))
            }
            _ => None,
        })
        .collect();
    let Some(last) = declared.last() else {
        return append_replacement(path, content, declaration);
    };

    let sorted = declared.windows(2).all(|pair| pair[0].0 <= pair[1].0);
    let before = declared
        .iter()
        .find(|(existing, _, _)| sorted && existing.as_str() > name);
    let at = match before {
        Some((_, start, _)) => *start,
        None => last.2,
    };
    // After the last line of the block the file may end without a newline.
    let prefix = if at == content.len() && !content.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    TextReplacement {
        path: path.to_path_buf(),
        start: at,
        end: at,
        replacement: format!("{prefix}{declaration}\n"),
    }
}

fn line_start(content: &str, offset: usize) -> usize {
    content[..offset].rfind('\n').map_or(0, |index| index + 1)
}

/// The offset just after the newline that ends the line holding `offset`.
fn line_end(content: &str, offset: usize) -> usize {
    content[offset..]
        .find('\n')
        .map_or(content.len(), |found| offset + found + 1)
}

#[cfg(test)]
mod tests;
