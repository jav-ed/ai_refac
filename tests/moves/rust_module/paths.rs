//! How the paths that name the module are rewritten: super, generic arguments, cfg(test) code, macro arguments, locals.

use super::{run, write};
use crate::common;

/// Real modules use generic types (`Vec<T>`, `Result<T, E>`), so a path with generic
/// arguments must not stop the move, and a `super::` path with generic arguments after
/// it keeps them: only its leading `super::` segments are rewritten.
#[test]
fn a_moved_file_with_generic_types_keeps_them_while_super_paths_are_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod consumer;\npub mod domain;\npub mod engine;\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(
        root,
        "src/engine/mod.rs",
        "pub mod matching;\npub mod shapes;\n",
    );
    write(
        root,
        "src/engine/shapes.rs",
        "pub struct Wrapper<T>(pub T);\npub fn make() -> Wrapper<u32> { Wrapper(7) }\n",
    );
    write(
        root,
        "src/engine/matching.rs",
        "use std::collections::HashMap;\n\npub fn execute(items: Vec<u32>) -> Result<HashMap<u32, Vec<String>>, String> {\n    let wrapped: super::shapes::Wrapper<Vec<u32>> = super::shapes::Wrapper(items);\n    Ok(HashMap::from([(wrapped.0.len() as u32, Vec::from([super::shapes::make().0.to_string()]))]))\n}\n",
    );
    write(
        root,
        "src/consumer.rs",
        "use crate::engine::matching::execute;\npub fn consume() -> usize { execute(vec![1]).unwrap().len() }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let moved = common::read_file(root, "src/domain/matching.rs");
    assert!(
        moved.contains("Result<HashMap<u32, Vec<String>>, String>"),
        "{moved}"
    );
    assert!(
        moved.contains(
            "crate::engine::shapes::Wrapper<Vec<u32>> = crate::engine::shapes::Wrapper(items)"
        ),
        "{moved}"
    );
    assert!(!moved.contains("super::"), "{moved}");
    let consumer = common::read_file(root, "src/consumer.rs");
    assert!(
        consumer.contains("crate::domain::matching::execute"),
        "{consumer}"
    );
}

/// Tests are part of the crate: a `#[cfg(test)]` module that uses the moved module, and one
/// inside the moved file that reaches back out of it, are rewritten like any other code.
#[test]
fn references_inside_cfg_test_code_are_rewritten_too() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\n\n#[cfg(test)]\nmod tests;\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n\n#[cfg(test)]\nmod tests {\n    use crate::engine::matching::*;\n    #[test]\n    fn reads() { assert_eq!(value(), 7); }\n}\n",
    );
    write(
        root,
        "src/tests.rs",
        "use crate::engine::matching::value;\n#[test]\nfn outside() { assert_eq!(value(), 7); }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let outside = common::read_file(root, "src/tests.rs");
    assert!(
        outside.contains("use crate::domain::matching::value;"),
        "{outside}"
    );
    let moved = common::read_file(root, "src/domain/matching.rs");
    assert!(moved.contains("use crate::domain::matching::*;"), "{moved}");
}

/// A `super` that stays inside the moved module is position independent: the `use super::*`
/// of an inline test module and a child module reaching its parent keep working as written,
/// while a `super` that leaves the module is made absolute.
#[test]
fn super_paths_inside_the_moved_module_stay_and_those_leaving_it_become_absolute() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\npub mod shared;\n",
    );
    write(root, "src/shared.rs", "pub fn base() -> u32 { 1 }\n");
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub mod child;\n\npub fn helper() -> u32 { 2 }\npub fn run() -> u32 { super::super::shared::base() + child::read() }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn runs() { assert_eq!(run(), 3); }\n}\n",
    );
    write(
        root,
        "src/engine/matching/child.rs",
        "use super::helper;\npub fn read() -> u32 { helper() }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let moved = common::read_file(root, "src/domain/matching.rs");
    assert!(moved.contains("use super::*;"), "{moved}");
    assert!(moved.contains("crate::shared::base()"), "{moved}");
    let child = common::read_file(root, "src/domain/matching/child.rs");
    assert!(child.contains("use super::helper;"), "{child}");
}

/// Paths in macro arguments are not parsed as paths; the ones that start at `crate` are found
/// by their tokens, in `assert_eq!`, `vec!` and nested macros.
#[test]
fn crate_paths_inside_macro_arguments_are_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\n\n#[cfg(test)]\nmod tests;\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub struct Item { pub a: u32 }\npub fn value() -> u32 { 7 }\n",
    );
    write(
        root,
        "src/tests.rs",
        "#[test]\nfn t() {\n    let items = vec![crate::engine::matching::Item { a: crate::engine::matching::value() }];\n    assert_eq!(items.len(), 1);\n    assert_eq!(crate::engine::matching::value(), 7);\n}\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let tests = common::read_file(root, "src/tests.rs");
    assert!(!tests.contains("crate::engine"), "{tests}");
    assert_eq!(
        tests.matches("crate::domain::matching::").count(),
        3,
        "{tests}"
    );
}

#[test]
fn a_module_path_inside_generic_arguments_is_rewritten_on_its_own() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\npub mod user;\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub struct Item { pub a: u32 }\n",
    );
    write(
        root,
        "src/user.rs",
        "use crate::engine;\n\npub fn all(items: &[engine::matching::Item]) -> Vec<crate::engine::matching::Item> {\n    let _: Option<Vec<engine::matching::Item>> = None;\n    items.iter().map(|item| crate::engine::matching::Item { a: item.a }).collect()\n}\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let user = common::read_file(root, "src/user.rs");
    assert!(!user.contains("engine::matching"), "{user}");
    assert_eq!(
        user.matches("crate::domain::matching::Item").count(),
        4,
        "{user}"
    );
}

#[test]
fn a_local_variable_named_like_the_module_is_left_alone() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod domain;\npub mod engine;\n");
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\nmod user;\n");
    write(
        root,
        "src/engine/matching.rs",
        "#[derive(Default)]\npub struct Found { pub items: Vec<u32> }\npub fn collect() -> Result<Found, String> { Ok(Found::default()) }\n",
    );
    write(
        root,
        "src/engine/user.rs",
        "use super::matching;\n\npub fn run() -> Result<usize, String> {\n    let matching = matching::collect()?;\n    let count = matching.items.len();\n    drop(matching);\n    Ok(count)\n}\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let user = common::read_file(root, "src/engine/user.rs");
    assert!(
        user.contains("let matching = matching::collect()?;"),
        "{user}"
    );
    assert!(user.contains("matching.items.len()"), "{user}");
    assert!(user.contains("drop(matching)"), "{user}");
}
