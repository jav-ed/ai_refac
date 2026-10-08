//! Paths to the moved module inside the arguments of a macro call or an
//! attribute: `assert_eq!(engine::matching::value(), 7)`,
//! `vec![super::matching::Item]`, `assert_eq!(self::matching::f(), 1)`.
//!
//! A token tree is not parsed as paths, and rust-analyzer lists the references
//! inside it only when it can expand the macro (`assert_eq!` and `vec!` come
//! from the standard library, which a loaded workspace does not have). So the
//! tokens are read here: every `word::word::…` chain is cut at the module's
//! name, and rust-analyzer's own name resolution, asked in the scope of the
//! macro call, decides whether that prefix is the moved module. A path through
//! `super`, `self`, an import, a glob or a nested module resolves like any
//! other; one that names a different module with the same name is left alone.
//! The rewrite is the one a path outside a macro gets: the short form through
//! an import stays short, any other spelling becomes the absolute path.

use super::imports::imports_module_by_name;
use super::macro_paths::{Piece, outermost_token_trees, pieces_of};
use super::new_path::desired_reference_path;
use crate::drivers::rust::analysis::module_graph::{self, ResolvedModule};
use crate::drivers::rust::analysis::workspace::SemanticWorkspace;
use crate::drivers::rust::transaction::apply::TextReplacement;
use anyhow::{Context, Result};
use ra_ap_hir::{Crate, ModuleDef, PathResolution, Semantics, SemanticsScope};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, Edition, SourceFile, ast, ast::make};
use std::{collections::HashSet, path::Path};

/// The edits for the macro arguments of every file of every crate of the
/// workspace; the module is reached from other crates by their crate name.
pub fn macro_reference_edits(
    workspace: &SemanticWorkspace,
    source: &ResolvedModule,
    target: &[String],
) -> Result<Vec<TextReplacement>> {
    let database = workspace.database();
    let semantics = Semantics::new(database);
    let mut edits = Vec::new();
    for krate in Crate::all(database) {
        if !workspace.is_local_crate(krate)? {
            continue;
        }
        let same_crate = krate == source.krate;
        for (file_id, path) in module_graph::crate_source_files(workspace, krate)? {
            let file = semantics.parse_guess_edition(file_id);
            for tree in outermost_token_trees(&file) {
                edits.extend(tree_edits(
                    &semantics, &path, &file, &tree, source, target, same_crate,
                )?);
            }
        }
    }
    // A file that belongs to two crates is visited twice.
    let mut seen = HashSet::new();
    edits.retain(|edit| seen.insert((edit.path.clone(), edit.start, edit.end)));
    Ok(edits)
}

fn tree_edits(
    semantics: &Semantics<'_, RootDatabase>,
    path: &Path,
    file: &SourceFile,
    tree: &ast::TokenTree,
    source: &ResolvedModule,
    target: &[String],
    same_crate: bool,
) -> Result<Vec<TextReplacement>> {
    let name = source
        .segments
        .last()
        .context("Source module has no name")?;
    let pieces = pieces_of(tree);
    // Most token trees (derives, doc attributes, `println!`) never mention the
    // module's name; asking for their scope is the costly part.
    let candidates: Vec<Chain> = chains(&pieces)
        .into_iter()
        .filter(|chain| {
            (chain.start..chain.start + chain.len)
                .step_by(2)
                .any(|index| pieces[index].text == *name)
        })
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let Some(scope) = semantics.scope(tree.syntax()) else {
        return Ok(Vec::new());
    };
    let mut edits = Vec::new();
    for chain in candidates {
        // The words of the chain sit at every second piece: `a :: b :: c`.
        let Some(at) = (chain.start..chain.start + chain.len)
            .step_by(2)
            .find(|&index| {
                pieces[index].text == *name
                    && names_the_module(&scope, &pieces[chain.start..=index], source)
            })
        else {
            continue;
        };
        let leaf: Vec<String> = pieces[chain.start..=at]
            .iter()
            .filter(|piece| piece.text != "::")
            .map(|piece| piece.text.clone())
            .collect();
        let range = pieces[chain.start].range.cover(pieces[at].range);

        // `name::item` in a file that imports the module as `name` keeps its
        // short form: the import is rewritten and brings the new name.
        if same_crate && leaf.len() == 1 && imports_module_by_name(file, &leaf[0]) {
            let new_name = target.last().context("Target module has no name")?;
            if *new_name != leaf[0] {
                edits.push(TextReplacement::from_range(
                    path.to_path_buf(),
                    range,
                    new_name.clone(),
                ));
            }
            continue;
        }
        if let Some(desired) = desired_reference_path(&leaf, &source.segments, target, same_crate)?
        {
            edits.push(TextReplacement::from_range(
                path.to_path_buf(),
                range,
                desired.join("::"),
            ));
        }
    }
    Ok(edits)
}

/// A run of pieces `word :: word :: … :: word` and where it starts.
struct Chain {
    start: usize,
    len: usize,
}

fn chains(pieces: &[Piece]) -> Vec<Chain> {
    let mut found = Vec::new();
    let mut index = 0;
    while index < pieces.len() {
        if !is_word(&pieces[index].text) {
            index += 1;
            continue;
        }
        let start = index;
        while pieces
            .get(index + 1)
            .is_some_and(|piece| piece.text == "::")
            && pieces
                .get(index + 2)
                .is_some_and(|piece| is_word(&piece.text))
        {
            index += 2;
        }
        // `::name` and `<T>::name` start somewhere this reader cannot see.
        let continues_something = start > 0 && pieces[start - 1].text == "::";
        if index > start && !continues_something {
            found.push(Chain {
                start,
                len: index - start + 1,
            });
        }
        index += 1;
    }
    found
}

fn is_word(text: &str) -> bool {
    !text.is_empty()
        && !text.starts_with(|character: char| character.is_ascii_digit())
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
}

/// Whether the path spelled by `prefix` (words and `::`) is the moved module
/// when written where the macro is called.
fn names_the_module(scope: &SemanticsScope<'_>, prefix: &[Piece], source: &ResolvedModule) -> bool {
    let text: String = prefix.iter().map(|piece| piece.text.as_str()).collect();
    let path = make::path_from_text_with_edition(&text, Edition::CURRENT);
    matches!(
        scope.speculative_resolve(&path),
        Some(PathResolution::Def(ModuleDef::Module(module))) if module == source.module
    )
}

#[cfg(test)]
mod tests;
