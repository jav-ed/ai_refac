//! Absolute `crate::…` paths inside the arguments of a macro call
//! (`assert_eq!(crate::a::b::f(), 1)`, `vec![crate::a::b::Item { .. }]`), read
//! from the tokens alone. This is the fallback behind `macro_references`,
//! which resolves every spelling through rust-analyzer; it covers the files
//! and macros where rust-analyzer cannot give a scope for the call. A path
//! through an import, `super` or another crate's name cannot be told from other
//! tokens without that resolution, so only the form that starts with `crate`
//! is recognised here.

use crate::drivers::rust::transaction::apply::TextReplacement;
use ra_ap_syntax::{
    AstNode, Edition, NodeOrToken, SourceFile, SyntaxKind, SyntaxToken, TextRange, ast,
};
use std::path::Path;

/// One word or punctuation of a token tree. rust-analyzer keeps `::` inside a
/// token tree as two `:` tokens, so adjacent colons are joined here.
pub(super) struct Piece {
    pub text: String,
    pub range: TextRange,
}

/// The edits that rewrite `crate::<source…>` to `crate::<target…>` in the
/// macro arguments of one file of the crate that owns the module.
pub fn macro_path_edits(
    path: &Path,
    content: &str,
    source: &[String],
    target: &[String],
) -> Vec<TextReplacement> {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    let replacement = format!("crate::{}", target.join("::"));
    let mut edits = Vec::new();
    for tree in outermost_token_trees(&parse.tree()) {
        let pieces = pieces_of(&tree);
        let mut index = 0;
        while index < pieces.len() {
            match matches_at(&pieces, index, source) {
                Some(last) => {
                    edits.push(TextReplacement::from_range(
                        path.to_path_buf(),
                        pieces[index].range.cover(pieces[last].range),
                        replacement.clone(),
                    ));
                    index = last + 1;
                }
                None => index += 1,
            }
        }
    }
    edits
}

/// The token trees that are not inside another one: the outermost holds the
/// tokens of the nested ones as well.
pub(super) fn outermost_token_trees(file: &ast::SourceFile) -> Vec<ast::TokenTree> {
    file.syntax()
        .descendants()
        .filter_map(ast::TokenTree::cast)
        .filter(|tree| {
            !tree
                .syntax()
                .ancestors()
                .skip(1)
                .any(|ancestor| ast::TokenTree::cast(ancestor).is_some())
        })
        .collect()
}

pub(super) fn pieces_of(tree: &ast::TokenTree) -> Vec<Piece> {
    let tokens: Vec<SyntaxToken> = tree
        .syntax()
        .descendants_with_tokens()
        .filter_map(NodeOrToken::into_token)
        .filter(|token| !is_trivia(token))
        .collect();
    let mut pieces: Vec<Piece> = Vec::new();
    for token in tokens {
        let joins_colon = token.text() == ":"
            && pieces.last().is_some_and(|last| {
                last.text == ":" && last.range.end() == token.text_range().start()
            });
        if joins_colon {
            let last = pieces.last_mut().expect("checked above");
            last.text = "::".to_string();
            last.range = last.range.cover(token.text_range());
        } else {
            pieces.push(Piece {
                text: token.text().to_string(),
                range: token.text_range(),
            });
        }
    }
    pieces
}

/// The index of the last piece of `crate :: s0 :: s1 …` starting at `index`.
fn matches_at(pieces: &[Piece], index: usize, source: &[String]) -> Option<usize> {
    if pieces[index].text != "crate" {
        return None;
    }
    // `other::crate::…` is not a path from the crate root.
    if index > 0 && pieces[index - 1].text == "::" {
        return None;
    }
    let mut at = index;
    for segment in source {
        if pieces.get(at + 1)?.text != "::" || pieces.get(at + 2)?.text != *segment {
            return None;
        }
        at += 2;
    }
    // A longer segment (`matchingx`) is a different name, and it is one token.
    Some(at)
}

fn is_trivia(token: &SyntaxToken) -> bool {
    matches!(token.kind(), SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
}

#[cfg(test)]
mod tests;
