//! Operator functions a file calls without writing their names: `money[i]`
//! (`get`), `for (x in bag)` (`iterator`, `hasNext`, `next`), `val (a, b) = p`
//! (`component1`, `component2`) and `val x by registry` (`getValue`,
//! `setValue`, `provideDelegate`). When such a function is a top-level
//! extension of the old package, the file needs an import for it, and the
//! language server does not add one: it imports the operators written as
//! symbols (`+`, `<`, `in`, `+=`, `!`, `..`) but not these (found on the real
//! server, 2026-10-12). Like the rest of this layer it errs towards naming too
//! much: an unused import is harmless and a missing one is a build error.

use super::used::Used;
use crate::drivers::kotlin::declarations::top_level::Declaration;
use std::collections::BTreeSet;

fn add(found: &mut BTreeSet<String>, names: &[&str]) {
    found.extend(names.iter().map(|name| name.to_string()));
}

/// A word that is a hard or soft keyword with an operator behind it. `at` is
/// the position after the word.
pub(super) fn keyword(word: &str, chars: &[char], at: usize, found: &mut BTreeSet<String>) {
    match word {
        "for" => {
            add(found, &["iterator", "hasNext", "next"]);
            // `for ((a, b) in pairs)`: the list is inside the header.
            let header = skip_blanks(chars, at);
            if chars.get(header) == Some(&'(') {
                let list = skip_blanks(chars, header + 1);
                if chars.get(list) == Some(&'(') {
                    components(chars, list, found);
                }
            }
        }
        "val" | "var" => {
            let list = skip_blanks(chars, at);
            if chars.get(list) == Some(&'(') {
                components(chars, list, found);
            }
        }
        "by" => add(found, &["getValue", "setValue", "provideDelegate"]),
        _ => {}
    }
}

fn skip_blanks(chars: &[char], mut at: usize) -> usize {
    while chars.get(at).is_some_and(|ch| ch.is_whitespace()) {
        at += 1;
    }
    at
}

/// `[` after a name, a closing bracket or a string is an index: reading it
/// calls `get`, assigning to it (`a[1] = 2`) calls `set`, and `a[1] += 2` or
/// `a[1]++` calls both.
pub(super) fn index(chars: &[char], open: usize, before: char, found: &mut BTreeSet<String>) {
    if !matches!(before, 'a' | ')' | ']' | '"') {
        return;
    }
    match after_index(chars, open) {
        Use::Read => add(found, &["get"]),
        Use::Assign => add(found, &["set"]),
        Use::Update => add(found, &["get", "set"]),
    }
}

/// Whether `used`, the names a file writes, reaches the operator extension
/// `declaration` through the syntax. The syntax alone cannot tell whose
/// `[i]` or `by` it is, and `by lazy` or `list[0]` are everywhere, so the
/// file has to write the type the extension is called on (a type parameter as
/// receiver is taken to match any file).
pub(super) fn reaches(declaration: &Declaration, used: &Used) -> bool {
    if !declaration.extension
        || !declaration.operator
        || !used.convention.contains(&declaration.name)
    {
        return false;
    }
    match declaration.receiver.as_deref() {
        Some(receiver) if receiver.chars().count() > 1 => {
            used.plain.contains(receiver) || used.member.contains(receiver)
        }
        _ => true,
    }
}

/// The operator functions the syntax can call without their names; the ones
/// a move has to import for a file that stayed behind.
pub(super) fn is_convention(name: &str) -> bool {
    matches!(
        name,
        "get"
            | "set"
            | "iterator"
            | "hasNext"
            | "next"
            | "getValue"
            | "setValue"
            | "provideDelegate"
    ) || name
        .strip_prefix("component")
        .is_some_and(|n| !n.is_empty() && n.chars().all(|ch| ch.is_ascii_digit()))
}

/// `{ (key, value) -> ... }`: a parenthesized list before `->` that follows
/// the brace or a comma of a lambda's parameters.
pub(super) fn lambda_parameters(
    chars: &[char],
    open: usize,
    before: char,
    found: &mut BTreeSet<String>,
) {
    if !matches!(before, '{' | ',') {
        return;
    }
    let Some(close) = closing(chars, open) else {
        return;
    };
    let mut rest = chars[close + 1..]
        .iter()
        .skip_while(|ch| ch.is_whitespace());
    if rest.next() == Some(&'-') && rest.next() == Some(&'>') {
        components(chars, open, found);
    }
}

/// `component1` to `componentN` for the N elements of the list that opens at
/// `open`.
fn components(chars: &[char], open: usize, found: &mut BTreeSet<String>) {
    let Some(close) = closing(chars, open) else {
        return;
    };
    let (mut depth, mut count) = (0usize, 1usize);
    for ch in &chars[open + 1..close] {
        match ch {
            '(' | '<' | '[' => depth += 1,
            ')' | '>' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    found.extend((1..=count).map(|n| format!("component{n}")));
}

/// The position of the bracket that closes the one at `open`.
fn closing(chars: &[char], open: usize) -> Option<usize> {
    let (opening, closing) = match chars.get(open)? {
        '(' => ('(', ')'),
        '[' => ('[', ']'),
        _ => return None,
    };
    let mut depth = 0usize;
    for (offset, ch) in chars[open..].iter().enumerate() {
        if *ch == opening {
            depth += 1;
        } else if *ch == closing {
            depth -= 1;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

enum Use {
    Read,
    Assign,
    Update,
}

/// What follows the index that opens at `open`.
fn after_index(chars: &[char], open: usize) -> Use {
    let Some(close) = closing(chars, open) else {
        return Use::Read;
    };
    let mut rest = chars[close + 1..]
        .iter()
        .skip_while(|ch| ch.is_whitespace());
    match (rest.next(), rest.next()) {
        (Some('='), next) if next != Some(&'=') => Use::Assign,
        (Some('+' | '-'), Some('=' | '+' | '-')) => Use::Update,
        (Some('*' | '/' | '%'), Some('=')) => Use::Update,
        _ => Use::Read,
    }
}
