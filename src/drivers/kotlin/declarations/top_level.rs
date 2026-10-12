//! The top-level declarations of a Kotlin file, read line by line (they start
//! in column 0).

const BOM: char = '\u{FEFF}';

/// What a top-level declaration is, as far as an import cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Class, interface, object, typealias: one name, one meaning.
    Type,
    /// Functions and properties: the same name can be declared many times
    /// (overloads, and `expect` with its `actual`).
    Callable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub name: String,
    pub kind: Kind,
    /// `fun Helper.undecorate()`: used as `helper.undecorate()`, so a use
    /// after a dot counts.
    pub extension: bool,
}

const MODIFIERS: &[&str] = &[
    "public",
    "internal",
    "protected",
    "open",
    "abstract",
    "final",
    "sealed",
    "data",
    "enum",
    "annotation",
    "inner",
    "value",
    "inline",
    "noinline",
    "crossinline",
    "suspend",
    "operator",
    "infix",
    "tailrec",
    "const",
    "lateinit",
    "expect",
    "actual",
    "external",
];

/// The visible (not `private`) declarations at the top level of a Kotlin
/// file. They start in column 0; annotations may come on the line before or
/// before the keyword on the same line.
pub fn declarations(text: &str) -> Vec<Declaration> {
    let mut found = Vec::new();
    let mut block_depth = 0usize;
    let mut raw = false;
    for line in text.trim_start_matches(BOM).lines() {
        let inside_literal = block_depth > 0 || raw;
        track_literals(line, &mut block_depth, &mut raw);
        if inside_literal || line.starts_with(char::is_whitespace) || line.starts_with("//") {
            continue;
        }
        if let Some(declaration) = declaration(line) {
            found.push(declaration);
        }
    }
    found
}

/// Follows block comments and raw strings across lines, so that a line in
/// the middle of either is never read as a declaration. Ordinary strings and
/// characters are skipped, so a `"/*"` in one does not open a comment.
fn track_literals(line: &str, block_depth: &mut usize, raw: &mut bool) {
    let chars: Vec<char> = line.chars().collect();
    let at = |i: usize, text: &str| {
        text.chars()
            .enumerate()
            .all(|(offset, ch)| chars.get(i + offset) == Some(&ch))
    };
    let mut i = 0;
    while i < chars.len() {
        if *raw {
            if at(i, "\"\"\"") {
                *raw = false;
                i += 3;
            } else {
                i += 1;
            }
        } else if *block_depth > 0 {
            if at(i, "/*") {
                *block_depth += 1;
                i += 2;
            } else if at(i, "*/") {
                *block_depth -= 1;
                i += 2;
            } else {
                i += 1;
            }
        } else if at(i, "//") {
            return;
        } else if at(i, "/*") {
            *block_depth += 1;
            i += 2;
        } else if at(i, "\"\"\"") {
            *raw = true;
            i += 3;
        } else if chars[i] == '"' {
            i = after_string(&chars, i);
        } else if chars[i] == '\'' {
            i = after_character(&chars, i);
        } else {
            i += 1;
        }
    }
}

/// The index after the string that starts at `start`, or the end of the line.
fn after_string(chars: &[char], start: usize) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            '"' => return i + 1,
            _ => i += 1,
        }
    }
    chars.len()
}

/// The index after the character literal that starts at `start` (`'a'`,
/// `'\n'`, `'\u0041'`); a lone apostrophe is one character.
fn after_character(chars: &[char], start: usize) -> usize {
    let escaped = chars.get(start + 1) == Some(&'\\');
    let search_from = start + if escaped { 3 } else { 2 };
    (search_from..chars.len().min(search_from + 6))
        .find(|i| chars[*i] == '\'')
        .map_or(start + 1, |i| i + 1)
}

fn declaration(line: &str) -> Option<Declaration> {
    let mut rest = line;
    loop {
        rest = rest.trim_start();
        if rest.starts_with('@') {
            rest = skip_annotation(rest)?;
            continue;
        }
        let (word, after) = split_word(rest);
        match word {
            "private" => return None,
            word if MODIFIERS.contains(&word) => rest = after,
            "fun" => {
                let (next, name_part) = split_word(after.trim_start());
                if next == "interface" {
                    return type_named(name_part);
                }
                return callable_named(after);
            }
            "val" | "var" => return callable_named(after),
            "class" | "interface" | "object" | "typealias" => return type_named(after),
            _ => return None,
        }
    }
}

/// `@Name`, `@file:Name`, `@Name(...)` with the arguments on this line.
fn skip_annotation(text: &str) -> Option<&str> {
    let end = text[1..]
        .find(|ch: char| !(ch.is_alphanumeric() || matches!(ch, '_' | '.' | ':')))
        .map_or(text.len(), |at| at + 1);
    let rest = &text[end..];
    if !rest.starts_with('(') {
        return Some(rest);
    }
    let mut depth = 0usize;
    for (at, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&rest[at + 1..]);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_word(text: &str) -> (&str, &str) {
    let end = text
        .find(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
        .unwrap_or(text.len());
    text.split_at(end)
}

fn type_named(text: &str) -> Option<Declaration> {
    let (name, _) = split_word(text.trim_start());
    (!name.is_empty()).then(|| Declaration {
        name: name.to_string(),
        kind: Kind::Type,
        extension: false,
    })
}

/// The name of a function or property after its keyword: the type parameters
/// (`<T>`) and the receiver (`List<T>.`, `String?.`) come first and are not
/// the name.
fn callable_named(text: &str) -> Option<Declaration> {
    let mut depth = 0i32;
    let mut current = String::new();
    let mut extension = false;
    let mut chars = text.trim_start().chars();
    while let Some(ch) = chars.next() {
        match ch {
            '<' => depth += 1,
            '>' if depth > 0 => depth -= 1,
            _ if depth > 0 => {}
            '`' => {
                current.clear();
                current.extend(chars.by_ref().take_while(|ch| *ch != '`'));
            }
            '.' => {
                extension = true;
                current.clear();
            }
            '?' => {}
            ch if ch.is_alphanumeric() || ch == '_' => current.push(ch),
            ch if ch.is_whitespace() && current.is_empty() => {}
            _ => break,
        }
    }
    (!current.is_empty()).then_some(Declaration {
        name: current,
        kind: Kind::Callable,
        extension,
    })
}
