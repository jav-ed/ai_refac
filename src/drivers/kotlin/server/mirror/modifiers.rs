//! `expect` and `actual` in the mirror.
//!
//! The mirror is one plain JVM module, so an `expect` declaration in it is a
//! declaration "in a platform module", and the server refuses to move it:
//! `Expected declaration 'function f()' would be moved to a platform module`.
//! The two modifiers are blanked in the mirror's copy of a file, replaced by
//! spaces of the same length, so that every line and column of the mirror is
//! still the line and column of the real file and the server's edits can be
//! applied to the real project as they are. An `expect` and its `actual` are
//! then two declarations of one name, which the server moves like any other.

/// What may stand between a modifier and the declaration it belongs to, or
/// start the declaration.
const KEYWORDS: &[&str] = &[
    "class",
    "interface",
    "object",
    "fun",
    "val",
    "var",
    "typealias",
    "constructor",
    "companion",
    "enum",
    "annotation",
    "data",
    "sealed",
    "open",
    "abstract",
    "final",
    "inner",
    "value",
    "inline",
    "suspend",
    "operator",
    "infix",
    "tailrec",
    "external",
    "const",
    "lateinit",
    "override",
    "internal",
    "private",
    "public",
    "protected",
];

/// `text` with the `expect` and `actual` modifiers of its declarations blanked.
pub fn blank_expect_actual(text: &str) -> String {
    text.split_inclusive('\n').map(blank_line).collect()
}

/// The first line (counted from 1) that declares something `expect` or
/// `actual`.
pub fn first_expect_actual_line(text: &str) -> Option<usize> {
    text.split_inclusive('\n')
        .position(|line| blank_line(line) != line)
        .map(|index| index + 1)
}

/// The first line (counted from 1) that declares `symbol` as something
/// `expect` or `actual`.
pub fn first_declaration_of(text: &str, symbol: &str) -> Option<usize> {
    let names = |line: &str| {
        line.split(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
            .any(|word| word == symbol)
    };
    text.split_inclusive('\n')
        .position(|line| blank_line(line) != line && names(line))
        .map(|index| index + 1)
}

/// Only a modifier counts: the word comes at the start of a line, after other
/// modifiers and annotations, and a declaration keyword follows it.
/// `assertEquals(expected, actual)`, `val actual = 1` and `expect(5)` stay.
fn blank_line(line: &str) -> String {
    let mut result = line.to_string();
    let mut at = line.len() - line.trim_start().len();
    loop {
        let rest = &line[at..];
        let length = rest
            .find(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '@'))
            .unwrap_or(rest.len());
        if length == 0 {
            break;
        }
        let word = &rest[..length];
        let after = rest[length..].trim_start();
        let followed = rest[length..].starts_with(char::is_whitespace) && starts_declaration(after);
        if matches!(word, "expect" | "actual") && followed {
            result.replace_range(at..at + length, &" ".repeat(length));
        } else if !word.starts_with('@') && !KEYWORDS.contains(&word) {
            break;
        }
        at += rest.len() - after.len();
    }
    result
}

fn starts_declaration(text: &str) -> bool {
    let word: String = text
        .chars()
        .take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '@')
        .collect();
    KEYWORDS.contains(&word.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_modifiers_of_declarations_are_blanked_without_moving_anything() {
        let text = "package a\n\nexpect fun f(): String\nactual class B(val x: Int) {\n    actual fun g() = 1\n    actual constructor(y: Long) : this(1)\n}\npublic expect object C\n@Composable expect fun d()\ninternal actual val v: Int = 2\n";
        let blanked = blank_expect_actual(text);
        assert_eq!(blanked.len(), text.len());
        assert_eq!(
            blanked,
            "package a\n\n       fun f(): String\n       class B(val x: Int) {\n           fun g() = 1\n           constructor(y: Long) : this(1)\n}\npublic        object C\n@Composable        fun d()\ninternal        val v: Int = 2\n"
        );
    }

    #[test]
    fn the_first_line_with_a_modifier_is_found() {
        assert_eq!(
            first_expect_actual_line("package a\n\nexpect fun f()\nactual fun g()\n"),
            Some(3)
        );
        assert_eq!(
            first_expect_actual_line("package a\nfun f() = expect(1)\n"),
            None
        );
    }

    #[test]
    fn a_symbol_declared_expect_or_actual_is_found_by_name() {
        let text = "package a\n\nexpect fun platformName(): String\nfun other() = platformName()\nactual class Sink\n";
        assert_eq!(first_declaration_of(text, "platformName"), Some(3));
        assert_eq!(first_declaration_of(text, "Sink"), Some(5));
        assert_eq!(first_declaration_of(text, "other"), None);
        assert_eq!(first_declaration_of(text, "platform"), None);
    }

    #[test]
    fn the_words_as_names_stay() {
        for line in [
            "assertEquals(expected, actual)\n",
            "    val actual = 1\n",
            "    expect(5)\n",
            "    actual = 3\n",
            "// expect fun a()\n",
            " * actual class B\n",
            "    return actual fun\n",
            "expect\n",
        ] {
            assert_eq!(blank_expect_actual(line), line, "{line}");
        }
    }
}
