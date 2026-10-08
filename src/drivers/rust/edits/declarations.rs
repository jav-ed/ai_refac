//! Where a new `mod name;` line goes in the file of its parent, and what
//! visibility it carries there. Appending at the end of the file would leave
//! it after the items and the tests module; the declarations of a parent form
//! a block at its top, usually in alphabetical order, and the new one joins
//! that block.

use crate::drivers::rust::transaction::apply::{TextReplacement, append_replacement};
use ra_ap_syntax::{
    AstNode, Edition, SourceFile,
    ast::{self, HasModuleItem, HasName},
};
use std::path::Path;

/// The edit that adds `declaration` (the text of a `mod <name>;` item, with
/// any attributes) to the file `content`. The block is the first run of
/// consecutive out-of-line declarations; a `mod tests;` far below it is not
/// part of it. The new one goes before the first existing declaration that
/// sorts after `name`, or after the last one when the block is not sorted or
/// `name` sorts last. A file without declarations gets it at the end.
pub fn insert_module_declaration(
    path: &Path,
    content: &str,
    name: &str,
    declaration: &str,
) -> TextReplacement {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    let mut declared: Vec<(String, usize, usize)> = Vec::new();
    for item in parse.tree().items() {
        let declaration = match &item {
            ast::Item::Module(module) if module.item_list().is_none() => module,
            _ if declared.is_empty() => continue,
            _ => break,
        };
        let Some(declared_name) = declaration.name() else {
            continue;
        };
        let range = declaration.syntax().text_range();
        declared.push((
            declared_name.text().to_string(),
            line_start(content, usize::from(range.start())),
            line_end(content, usize::from(range.end())),
        ));
    }
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

/// The visibility, with a trailing space, that a declaration copied from the
/// parent `old_parent` to the parent `new_parent` needs so that everything
/// that could name the module before still can. A private `mod` (or
/// `pub(self)`) is visible in the module that declares it and below, and
/// `pub(super)` in that module's parent and below; in a deeper parent either
/// would vanish for its old users, so it is rewritten to the lowest module
/// that holds the old reach and the new parent. `pub`, `pub(crate)` and
/// `pub(in crate::…)` are written as they were.
pub fn visibility_prefix(written: &str, old_parent: &[String], new_parent: &[String]) -> String {
    let reach = match written {
        "" | "pub(self)" => old_parent,
        "pub(super)" => &old_parent[..old_parent.len().saturating_sub(1)],
        _ => return format!("{written} "),
    };
    let shared = reach
        .iter()
        .zip(new_parent)
        .take_while(|(old, new)| old == new)
        .count();
    if shared == new_parent.len() {
        String::new()
    } else if shared + 1 == new_parent.len() {
        "pub(super) ".to_string()
    } else if shared == 0 {
        "pub(crate) ".to_string()
    } else {
        format!("pub(in crate::{}) ", new_parent[..shared].join("::"))
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
