//! Which names a Kotlin file uses. Like `declarations.rs` this does not parse
//! the language; the server does that. It only has to be right about names,
//! and where it is unsure it errs towards naming too much, because an import
//! of a name the file does not use is harmless and a missing one is a build
//! error.

use std::collections::BTreeSet;

const BOM: char = '\u{FEFF}';

/// The names a file uses outside comments and string literals (but inside
/// string templates).
#[derive(Debug, Default)]
pub struct Used {
    /// Names written on their own: `shout(x)`, `Helper()`, `@Composable`.
    pub plain: BTreeSet<String>,
    /// Names written after a dot: `x.shout()`. Only an extension function
    /// needs an import for that.
    pub member: BTreeSet<String>,
}

pub fn used_names(text: &str) -> Used {
    // The words of the package and import lines are paths, not uses.
    let code: String = text
        .trim_start_matches(BOM)
        .split_inclusive('\n')
        .filter(|line| {
            let line = line.trim_start();
            !(line.starts_with("import ") || line.starts_with("package "))
        })
        .collect();
    let mut scanner = Scanner {
        chars: code.chars().collect(),
        at: 0,
        before: ' ',
        before_that: ' ',
        used: Used::default(),
    };
    scanner.code(false);
    scanner.used
}

struct Scanner {
    chars: Vec<char>,
    at: usize,
    /// The last two characters of code that were not blanks, for `.` and `..`.
    before: char,
    before_that: char,
    used: Used,
}

impl Scanner {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn starts_with(&self, text: &str) -> bool {
        text.chars()
            .enumerate()
            .all(|(offset, ch)| self.chars.get(self.at + offset) == Some(&ch))
    }

    fn mark(&mut self, ch: char) {
        self.before_that = self.before;
        self.before = ch;
    }

    /// Code up to the end of the text, or in a `${ ... }` template up to its
    /// closing brace.
    fn code(&mut self, in_template: bool) {
        let mut depth = 0usize;
        while let Some(ch) = self.peek() {
            if self.starts_with("//") {
                while self.peek().is_some_and(|ch| ch != '\n') {
                    self.at += 1;
                }
            } else if self.starts_with("/*") {
                self.block_comment();
            } else if ch == '"' {
                self.string();
                self.mark('"');
            } else if ch == '\'' {
                self.character();
                self.mark('\'');
            } else if ch == '`' {
                let name = self.quoted_name();
                self.record(name);
            } else if ch.is_ascii_digit() {
                self.number();
                self.mark('0');
            } else if ch.is_alphabetic() || ch == '_' {
                let name = self.word();
                self.record(name);
            } else if ch.is_whitespace() {
                self.at += 1;
            } else {
                self.at += 1;
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    if in_template && depth == 0 {
                        return;
                    }
                    depth = depth.saturating_sub(1);
                }
                self.mark(ch);
            }
        }
    }

    /// Block comments nest in Kotlin.
    fn block_comment(&mut self) {
        let mut depth = 0usize;
        while self.peek().is_some() {
            if self.starts_with("/*") {
                depth += 1;
                self.at += 2;
            } else if self.starts_with("*/") {
                depth -= 1;
                self.at += 2;
                if depth == 0 {
                    return;
                }
            } else {
                self.at += 1;
            }
        }
    }

    fn word(&mut self) -> String {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.at += 1;
        }
        self.chars[start..self.at].iter().collect()
    }

    fn quoted_name(&mut self) -> String {
        self.at += 1;
        let start = self.at;
        while self.peek().is_some_and(|ch| ch != '`') {
            self.at += 1;
        }
        let name = self.chars[start..self.at].iter().collect();
        self.at += 1;
        name
    }

    /// `1`, `0x1F`, `100L`, `1.5f`; a dot belongs to the number only when a
    /// digit follows it (`1.shout()` and `1..n` are not decimals).
    fn number(&mut self) {
        while let Some(ch) = self.peek() {
            let decimal_point = ch == '.'
                && self
                    .chars
                    .get(self.at + 1)
                    .is_some_and(char::is_ascii_digit);
            if ch.is_alphanumeric() || ch == '_' || decimal_point {
                self.at += 1;
            } else {
                break;
            }
        }
    }

    fn record(&mut self, name: String) {
        let after_dot = self.before == '.' && self.before_that != '.';
        if after_dot {
            self.used.member.insert(name);
        } else {
            self.used.plain.insert(name);
        }
        self.mark('a');
    }

    fn character(&mut self) {
        self.at += 1;
        while let Some(ch) = self.peek() {
            self.at += 1;
            match ch {
                '\\' => self.at += 1,
                '\'' | '\n' => return,
                _ => {}
            }
        }
    }

    /// A string or a raw string. `$name` and `${ expression }` inside it are
    /// code.
    fn string(&mut self) {
        let raw = self.starts_with("\"\"\"");
        self.at += if raw { 3 } else { 1 };
        while let Some(ch) = self.peek() {
            if raw && self.starts_with("\"\"\"") {
                self.at += 3;
                // A raw string may end in extra quotes: """a"""" is a"
                while self.peek() == Some('"') {
                    self.at += 1;
                }
                return;
            }
            match ch {
                '"' if !raw => {
                    self.at += 1;
                    return;
                }
                '\n' if !raw => return,
                '\\' if !raw => self.at += 2,
                '$' => self.template(),
                _ => self.at += 1,
            }
        }
    }

    fn template(&mut self) {
        self.at += 1;
        match self.peek() {
            Some('{') => {
                self.at += 1;
                let (before, before_that) = (self.before, self.before_that);
                self.mark('{');
                self.code(true);
                self.at += 1;
                self.before = before;
                self.before_that = before_that;
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let name = self.word();
                self.used.plain.insert(name);
            }
            _ => {}
        }
    }
}
