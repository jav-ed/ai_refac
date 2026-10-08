//! Looking for old class names that the move left behind in files refac does
//! not edit: ProGuard rules, build scripts (`mainClass`), service lists,
//! configuration, XML it could not rewrite, and string literals (reflection).
//! Nothing is changed; each hit is reported so the user can fix it.

use super::class_renames::Renames;
use super::survey::Survey;
use crate::drivers::lsp::rename::write::journal::FileWrite;
use anyhow::{Context, Result};
use std::path::Path;

/// `planned` holds changes that are decided but not written yet; a file in it
/// is read as it will be, not as it is on disk.
pub fn scan(
    root: &Path,
    survey: &Survey,
    renames: &Renames,
    planned: &[FileWrite],
) -> Result<Vec<String>> {
    let names: Vec<&(String, String)> = renames.classes.iter().chain(&renames.facades).collect();
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut notes = Vec::new();
    let text_files = survey.naming_text.iter().chain(&survey.android_xml);
    for (files, only_in_quotes) in [
        (text_files.collect::<Vec<_>>(), false),
        (survey.sources.iter().collect(), true),
    ] {
        for path in files {
            let text = match planned.iter().find(|write| write.path == *path) {
                Some(write) => String::from_utf8_lossy(&write.bytes).into_owned(),
                None => std::fs::read_to_string(path)
                    .with_context(|| format!("Cannot read {}", path.display()))?,
            };
            for (old, new) in &names {
                if mentions(&text, old, only_in_quotes) {
                    let shown = path.strip_prefix(root).unwrap_or(path).display();
                    notes.push(format!(
                        "{shown} still mentions {old} (now {new}); refac does not edit this kind of reference"
                    ));
                }
            }
        }
    }
    Ok(notes)
}

/// `name` as a whole word: not part of a longer name or qualified path. In
/// source files only a string literal counts, because real references there
/// were already updated by the server.
fn mentions(text: &str, name: &str, only_in_quotes: bool) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + name.len()..].chars().next();
        let starts_word = before.is_none_or(|ch| !(is_name_char(ch) || ch == '.'));
        let ends_word = after.is_none_or(|ch| !is_name_char(ch));
        let quoted = before.is_some_and(|ch| ch == '"' || ch == '\'');
        starts_word && ends_word && (!only_in_quotes || quoted)
    })
}

fn is_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_whole_names_only() {
        let text = "-keep class com.example.util.Helper { *; }\n";
        assert!(mentions(text, "com.example.util.Helper", false));
        assert!(!mentions(
            "com.example.util.HelperExtra",
            "com.example.util.Helper",
            false
        ));
        assert!(!mentions(
            "org.com.example.util.Helper",
            "com.example.util.Helper",
            false
        ));
    }

    #[test]
    fn in_sources_only_string_literals_count() {
        let reflection = "Class.forName(\"com.example.util.Helper\")";
        let reference = "val x = com.example.util.Helper(1)";
        assert!(mentions(reflection, "com.example.util.Helper", true));
        assert!(!mentions(reference, "com.example.util.Helper", true));
    }

    #[test]
    fn reports_file_old_and_new_name() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("build.gradle.kts");
        std::fs::write(
            &script,
            "application { mainClass.set(\"com.example.app.MainKt\") }\n",
        )
        .unwrap();
        let survey = Survey {
            naming_text: vec![script],
            ..Survey::default()
        };
        let renames = Renames {
            facades: vec![(
                "com.example.app.MainKt".to_string(),
                "com.example.cli.MainKt".to_string(),
            )],
            ..Renames::default()
        };

        let notes = scan(dir.path(), &survey, &renames, &[]).unwrap();
        assert_eq!(notes.len(), 1);
        assert!(notes[0].starts_with("build.gradle.kts still mentions com.example.app.MainKt (now com.example.cli.MainKt)"), "{}", notes[0]);

        // The same file, already planned to say the new name, is not stale.
        let fixed = FileWrite {
            path: dir.path().join("build.gradle.kts"),
            bytes: b"application { mainClass.set(\"com.example.cli.MainKt\") }\n".to_vec(),
            changes: 1,
        };
        assert!(
            scan(dir.path(), &survey, &renames, &[fixed])
                .unwrap()
                .is_empty()
        );
    }
}
