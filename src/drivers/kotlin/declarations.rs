//! A light reading of Kotlin source for the few facts refac checks itself:
//! the package a file declares and which types it declares at top level. It is line based on purpose; a
//! full parser lives in the language server, and these facts only back up
//! checks on the server's work.

const BOM: char = '\u{FEFF}';

const MODIFIERS: &[&str] = &[
    "public",
    "private",
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
    "expect",
    "actual",
    "fun",
];

/// The package a Kotlin file declares, `None` for the default package.
/// Comments, blank lines and `@file:` annotations may come before it.
pub fn declared_package(text: &str) -> Option<String> {
    let mut in_block_comment = false;
    for line in text.trim_start_matches(BOM).lines() {
        let line = line.trim();
        if in_block_comment {
            in_block_comment = !line.contains("*/");
            continue;
        }
        if line.is_empty() || line.starts_with("//") || line.starts_with("@file:") {
            continue;
        }
        if line.starts_with("/*") {
            in_block_comment = !line.contains("*/");
            continue;
        }
        let declaration = line.strip_prefix("package")?;
        if !declaration.starts_with(char::is_whitespace) {
            return None;
        }
        let name = declaration.split(['/', ';']).next()?.trim();
        return (!name.is_empty()).then(|| name.to_string());
    }
    None
}

/// Names of the classes, interfaces and objects declared at the top level,
/// in order. Top-level declarations start in column 0.
pub fn top_level_types(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_block_comment = false;
    for line in text.trim_start_matches(BOM).lines() {
        if in_block_comment {
            in_block_comment = !line.contains("*/");
            continue;
        }
        if line.starts_with("/*") {
            in_block_comment = !line.contains("*/");
            continue;
        }
        if line.starts_with(char::is_whitespace) || line.starts_with("//") {
            continue;
        }
        if let Some(name) = declared_type(line) {
            names.push(name);
        }
    }
    names
}

fn declared_type(line: &str) -> Option<String> {
    let mut tokens = line.split_whitespace();
    loop {
        let token = tokens.next()?;
        if token.starts_with('@') || MODIFIERS.contains(&token) {
            continue;
        }
        if !matches!(token, "class" | "interface" | "object") {
            return None;
        }
        let name: String = tokens
            .next()?
            .chars()
            .take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '`')
            .collect();
        return (!name.is_empty()).then_some(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_package_after_comments_and_file_annotations() {
        let text = "\u{FEFF}// header\n/* block\n package fake.one\n*/\n@file:JvmName(\"X\")\n\npackage com.example.util;\n\nimport a.B\n";
        assert_eq!(declared_package(text), Some("com.example.util".to_string()));
        assert_eq!(
            declared_package("package a.b // note\nclass A"),
            Some("a.b".to_string())
        );
    }

    #[test]
    fn a_file_without_a_package_is_in_the_default_package() {
        assert_eq!(declared_package("import a.B\nclass A\n"), None);
        assert_eq!(declared_package("class A\n"), None);
        assert_eq!(declared_package("packages()\n"), None);
        assert_eq!(declared_package(""), None);
    }

    #[test]
    fn reads_types_with_modifiers_and_annotations() {
        let text = "package a\n\nimport b.C\n\n@Suppress(\"X\") internal data class Point(val x: Int)\n\nsealed interface Shape\nenum class Color { RED }\nobject Registry {\n    class Nested\n}\nfun helper() {}\nfun interface Callback { fun call() }\n";
        assert_eq!(
            top_level_types(text),
            vec!["Point", "Shape", "Color", "Registry", "Callback"]
        );
    }

    #[test]
    fn ignores_comments_and_nested_declarations() {
        let text = "/* class Hidden\n class AlsoHidden */\n// class Commented\nclass Real {\n    class Inner\n}\n";
        assert_eq!(top_level_types(text), vec!["Real"]);
    }

    #[test]
    fn a_file_of_functions_declares_no_types() {
        assert!(top_level_types("package a\n\nfun shout(): String = \"\"\nval x = 1\n").is_empty());
    }
}
