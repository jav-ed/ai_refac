//! The class names that changed because files moved: what Android XML,
//! build scripts and string literals may still refer to by the old name.

use super::moved::MovedFile;
use crate::drivers::kotlin::declarations::{declared_package, top_level_types};
use anyhow::{Result, bail};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Renames {
    /// (old, new) fully qualified names of classes, interfaces and objects.
    pub classes: Vec<(String, String)>,
    /// (old, new) names of the `<File>Kt` classes that hold top-level functions
    /// and properties. Nothing in a project names them except build scripts,
    /// ProGuard rules and reflection, so they are only looked for, never edited.
    pub facades: Vec<(String, String)>,
}

pub fn collect(moved: &[MovedFile]) -> Result<Renames> {
    let mut renames = Renames::default();
    for file in moved {
        let (old_types, new_types) = (top_level_types(&file.before), top_level_types(&file.after));
        if old_types.len() != new_types.len() {
            bail!(
                "{} declared {} top-level types before the move and {} after it, so refac cannot tell which class became which",
                file.to.display(),
                old_types.len(),
                new_types.len()
            );
        }
        let old_package = declared_package(&file.before);
        let new_package = declared_package(&file.after);
        for (old, new) in old_types.iter().zip(&new_types) {
            push_if_changed(
                &mut renames.classes,
                qualified(old_package.as_deref(), old),
                qualified(new_package.as_deref(), new),
            );
        }
        push_if_changed(
            &mut renames.facades,
            qualified(old_package.as_deref(), &facade(&file.from)),
            qualified(new_package.as_deref(), &facade(&file.to)),
        );
    }
    Ok(renames)
}

fn push_if_changed(list: &mut Vec<(String, String)>, old: String, new: String) {
    if old != new {
        list.push((old, new));
    }
}

fn qualified(package: Option<&str>, name: &str) -> String {
    match package {
        Some(package) => format!("{package}.{name}"),
        None => name.to_string(),
    }
}

/// Kotlin names the class of a file's top-level members after the file:
/// `shout_it.kt` becomes `Shout_itKt`.
fn facade(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let mut chars = stem.chars();
    let first: String = chars
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .collect();
    format!("{first}{}Kt", chars.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn moved(from: &str, to: &str, before: &str, after: &str) -> MovedFile {
        MovedFile {
            from: PathBuf::from(from),
            to: PathBuf::from(to),
            before: before.to_string(),
            after: after.to_string(),
        }
    }

    #[test]
    fn a_package_change_renames_every_type_and_the_facade() {
        let files = [moved(
            "/p/util/Helper.kt",
            "/p/common/Helper.kt",
            "package com.example.util\n\nclass Helper\nobject Registry\nfun shout() {}\n",
            "package com.example.common\n\nclass Helper\nobject Registry\nfun shout() {}\n",
        )];
        let renames = collect(&files).unwrap();
        assert_eq!(
            renames.classes,
            [
                (
                    "com.example.util.Helper".to_string(),
                    "com.example.common.Helper".to_string()
                ),
                (
                    "com.example.util.Registry".to_string(),
                    "com.example.common.Registry".to_string()
                ),
            ]
        );
        assert_eq!(
            renames.facades,
            [(
                "com.example.util.HelperKt".to_string(),
                "com.example.common.HelperKt".to_string()
            )]
        );
    }

    #[test]
    fn a_class_renamed_with_its_file_maps_old_to_new() {
        let files = [moved(
            "/p/app/Greeter.kt",
            "/p/app/Welcomer.kt",
            "package a.app\n\nclass Greeter\n",
            "package a.app\n\nclass Welcomer\n",
        )];
        let renames = collect(&files).unwrap();
        assert_eq!(
            renames.classes,
            [("a.app.Greeter".to_string(), "a.app.Welcomer".to_string())]
        );
        assert_eq!(
            renames.facades,
            [(
                "a.app.GreeterKt".to_string(),
                "a.app.WelcomerKt".to_string()
            )]
        );
    }

    #[test]
    fn a_changed_number_of_types_is_an_error() {
        let files = [moved(
            "/p/A.kt",
            "/q/A.kt",
            "package a\nclass A\nclass B\n",
            "package q\nclass A\n",
        )];
        assert!(collect(&files).is_err());
    }
}
