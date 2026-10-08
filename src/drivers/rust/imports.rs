//! Whether a file brings a module into scope under its own name
//! (`use a::b::name;` or `use a::b::name::{self, Item};`). A path written
//! `name::item` in that file then reaches the module through the import, so
//! it can keep its short form when the import is rewritten.

use ra_ap_syntax::{AstNode, SourceFile, ast};

pub fn imports_module_by_name(file: &SourceFile, name: &str) -> bool {
    file.syntax()
        .descendants()
        .filter_map(ast::UseTree::cast)
        .any(|tree| {
            let Some(path) = tree.path() else {
                return false;
            };
            if last_segment(&path).as_deref() != Some(name) {
                return false;
            }
            match tree.use_tree_list() {
                // `use a::name;` (not renamed)
                None => tree.rename().is_none(),
                // `use a::name::{self, ...};`
                Some(list) => list.use_trees().any(|inner| {
                    inner.rename().is_none()
                        && inner.use_tree_list().is_none()
                        && inner.path().is_some_and(|inner_path| {
                            matches!(
                                inner_path.segment().and_then(|segment| segment.kind()),
                                Some(ast::PathSegmentKind::SelfKw)
                            )
                        })
                }),
            }
        })
}

fn last_segment(path: &ast::Path) -> Option<String> {
    Some(path.segment()?.name_ref()?.text().to_string())
}

#[cfg(test)]
mod tests;
