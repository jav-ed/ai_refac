//! Paths to the module inside the arguments of macro calls: they are tokens, not paths,
//! and rust-analyzer does not list them when it cannot expand the macro (`assert_eq!` and
//! `vec!` come from the standard library). `move-module` resolves them itself.

use super::{run, write};
use crate::common;

fn crate_manifest(root: &std::path::Path) {
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
}

/// Every spelling a path can have: through an import (kept short, the import is rewritten
/// and renamed), `super`, `self`, a nested path that starts at an imported parent, `$crate`
/// in a `macro_rules!` body, and `crate::`.
#[test]
fn super_self_imported_and_crate_paths_in_macro_arguments_are_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    crate_manifest(root);
    write(
        root,
        "src/lib.rs",
        r#"pub mod domain;
pub mod engine;

#[allow(unused_macros)]
macro_rules! call {
    () => { $crate::engine::matching::value() };
}

#[cfg(test)]
mod tests {
    use crate::engine;
    use crate::engine::matching;

    #[test]
    fn t() {
        assert_eq!(engine::matching::value(), 7);
        assert_eq!(super::engine::matching::value(), 7);
        assert_eq!(matching::value(), 7);
        assert_eq!(crate::engine::matching::value(), 7);
        assert_eq!(call!(), 7);
        let items = vec![matching::value(), engine::matching::value()];
        assert_eq!(items.len(), 2);
    }
}
"#,
    );
    write(root, "src/domain/mod.rs", "");
    write(
        root,
        "src/engine/mod.rs",
        r#"pub mod matching;

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        assert_eq!(super::matching::value(), 7);
        assert_eq!(super::inner(), 1);
    }
}

fn inner() -> u32 {
    assert_eq!(self::matching::value(), 7);
    1
}
"#,
    );
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );

    // The move itself runs `cargo check --all-targets` and undoes everything when a
    // path is stale, so a success here means every spelling compiles.
    let output = run(root, "crate::engine::matching", "crate::domain::found");
    common::assert_move_succeeded(&output);

    let lib = common::read_file(root, "src/lib.rs");
    assert!(lib.contains("$crate::domain::found::value()"), "{lib}");
    assert!(lib.contains("use crate::domain::found;"), "{lib}");
    assert!(lib.contains("assert_eq!(found::value(), 7);"), "{lib}");
    assert!(!lib.contains("matching"), "{lib}");
    let engine = common::read_file(root, "src/engine/mod.rs");
    assert!(!engine.contains("matching"), "{engine}");
    assert_eq!(
        engine.matches("crate::domain::found::value()").count(),
        2,
        "{engine}"
    );
}

/// A path through another module of the same name resolves to something else and stays.
#[test]
fn a_module_of_the_same_name_elsewhere_is_left_alone_in_macro_arguments() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    crate_manifest(root);
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\npub mod other;\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        assert_eq!(crate::other::matching::value(), 1);\n        assert_eq!(super::other::matching::value(), 1);\n        assert_eq!(crate::engine::matching::value(), 7);\n    }\n}\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );
    write(root, "src/other/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/other/matching.rs",
        "pub fn value() -> u32 { 1 }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let lib = common::read_file(root, "src/lib.rs");
    assert!(lib.contains("crate::other::matching::value()"), "{lib}");
    assert!(lib.contains("super::other::matching::value()"), "{lib}");
    assert!(lib.contains("crate::domain::matching::value()"), "{lib}");
    assert!(!lib.contains("engine::matching"), "{lib}");
}

/// Another crate of the workspace names the module by the crate's name.
#[test]
fn a_dependent_crate_names_the_module_by_the_crate_inside_macro_arguments() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"core\", \"app\"]\nresolver = \"3\"\n",
    );
    write(
        root,
        "core/Cargo.toml",
        "[package]\nname = \"core_lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "core/src/lib.rs", "pub mod matching;\n");
    write(
        root,
        "core/src/matching.rs",
        "pub fn value() -> u32 { 11 }\n",
    );
    write(
        root,
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[dependencies]\ncore_lib = { path = \"../core\" }\n",
    );
    write(
        root,
        "app/src/main.rs",
        "fn main() {\n    assert_eq!(core_lib::matching::value(), 11);\n    println!(\"{}\", format!(\"{}\", core_lib::matching::value()));\n}\n",
    );

    let output = run(root, "crate::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let app = common::read_file(root, "app/src/main.rs");
    assert_eq!(
        app.matches("core_lib::domain::matching::value()").count(),
        2,
        "{app}"
    );
}

/// The `super::` paths in the macro arguments of the moved module itself: the ones that leave
/// the module become absolute, the ones that stay inside it (a `mod tests` in the file) stay.
#[test]
fn super_paths_leaving_the_moved_module_become_absolute_inside_macro_arguments() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    crate_manifest(root);
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\n\npub fn top() -> u32 {\n    2\n}\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(
        root,
        "src/engine/mod.rs",
        "pub mod matching;\n\npub fn helper() -> u32 {\n    1\n}\n",
    );
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 {\n    7\n}\n\npub fn check() {\n    assert_eq!(super::helper(), 1);\n    assert_eq!(super::super::top(), 2);\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        assert_eq!(super::value(), 7);\n    }\n}\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::found");
    common::assert_move_succeeded(&output);

    let moved = common::read_file(root, "src/domain/found.rs");
    assert!(
        moved.contains("assert_eq!(crate::engine::helper(), 1);"),
        "{moved}"
    );
    assert!(moved.contains("assert_eq!(crate::top(), 2);"), "{moved}");
    assert!(moved.contains("assert_eq!(super::value(), 7);"), "{moved}");
}
