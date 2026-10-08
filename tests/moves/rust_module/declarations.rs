//! The `mod` line of the moved module: where it lands, which attributes it may carry, and how far it reaches.

use super::{run, write};
use crate::common;

/// The declaration of the moved module joins the block of declarations of its new parent,
/// in alphabetical order, instead of landing after the items and the tests.
#[test]
fn the_new_declaration_joins_the_parents_block_in_order() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod domain;\npub mod engine;\n");
    write(
        root,
        "src/domain/mod.rs",
        "pub mod alpha;\npub mod omega;\n\npub fn run() {}\n",
    );
    write(root, "src/domain/alpha.rs", "");
    write(root, "src/domain/omega.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    assert_eq!(
        common::read_file(root, "src/domain/mod.rs"),
        "pub mod alpha;\npub mod matching;\npub mod omega;\n\npub fn run() {}\n"
    );
}

/// The convention of this repository: every module has `#[cfg(test)] mod tests;` and its tests in
/// `<module>/tests.rs`. The test child moves with the module, and an attributed declaration of the
/// moved module itself keeps its attribute in the new parent.
#[test]
fn a_module_with_a_cfg_test_child_and_a_cfg_declaration_moves_whole() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod domain;\npub mod engine;\n");
    write(root, "src/domain/mod.rs", "");
    write(
        root,
        "src/engine/mod.rs",
        "#[cfg(test)]\npub mod matching;\n",
    );
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n\n#[cfg(test)]\nmod tests;\n",
    );
    write(
        root,
        "src/engine/matching/tests.rs",
        "use super::*;\n#[test]\nfn reads() { assert_eq!(value(), 7); }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    assert!(root.join("src/domain/matching.rs").exists());
    assert!(root.join("src/domain/matching/tests.rs").exists());
    assert!(!root.join("src/engine/matching/tests.rs").exists());
    assert_eq!(
        common::read_file(root, "src/domain/mod.rs"),
        "#[cfg(test)]\npub mod matching;\n"
    );
    assert_eq!(common::read_file(root, "src/engine/mod.rs"), "");
    assert_eq!(
        common::read_file(root, "src/domain/matching/tests.rs"),
        "use super::*;\n#[test]\nfn reads() { assert_eq!(value(), 7); }\n"
    );
}

/// `#[macro_use]` makes the order of declarations matter, so moving that declaration is refused,
/// naming the attribute, and nothing changes.
#[test]
fn a_macro_use_declaration_is_refused_by_name_and_changes_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod domain;\npub mod engine;\n");
    write(root, "src/domain/mod.rs", "");
    write(
        root,
        "src/engine/mod.rs",
        "#[macro_use]\npub mod matching;\n",
    );
    write(root, "src/engine/matching.rs", "pub fn value() {}\n");

    let output = run(root, "crate::engine::matching", "crate::domain::matching");

    assert!(!output.status.success());
    let error = common::stderr_text(&output);
    assert!(error.contains("macro_use"), "{error}");
    assert!(root.join("src/engine/matching.rs").exists());
    assert_eq!(
        common::read_file(root, "src/engine/mod.rs"),
        "#[macro_use]\npub mod matching;\n"
    );
}

#[test]
fn pub_super_keeps_its_reach_when_the_module_moves_deeper() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod engine;\n");
    write(
        root,
        "src/engine/mod.rs",
        "pub mod matching;\npub mod plan;\n\npub fn run() -> u32 {\n    matching::helper()\n}\n",
    );
    write(root, "src/engine/plan/mod.rs", "");
    write(
        root,
        "src/engine/matching.rs",
        "pub(super) fn helper() -> u32 { 7 }\npub(self) fn own() {}\npub(crate) fn wide() {}\npub(in super::super) fn explicit() {}\n",
    );

    let output = run(
        root,
        "crate::engine::matching",
        "crate::engine::plan::matching",
    );
    common::assert_move_succeeded(&output);

    let moved = common::read_file(root, "src/engine/plan/matching.rs");
    assert!(
        moved.contains("pub(in crate::engine) fn helper()"),
        "{moved}"
    );
    assert!(moved.contains("pub(self) fn own()"), "{moved}");
    assert!(moved.contains("pub(crate) fn wide()"), "{moved}");
    assert!(moved.contains("pub(in crate) fn explicit()"), "{moved}");
}

#[test]
fn a_private_module_moved_deeper_stays_visible_to_its_old_users() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod engine;\n");
    write(
        root,
        "src/engine/mod.rs",
        "mod matching;\nmod user;\n\npub fn run() -> u32 {\n    user::go() + matching::value()\n}\n\n#[cfg(test)]\nmod tests {}\n",
    );
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );
    write(
        root,
        "src/engine/user.rs",
        "pub fn go() -> u32 { super::matching::value() }\n",
    );

    let output = run(
        root,
        "crate::engine::matching",
        "crate::engine::plan::matching",
    );
    common::assert_move_succeeded(&output);

    let parent = common::read_file(root, "src/engine/mod.rs");
    assert!(
        parent.contains("\nmod plan;\n") || parent.starts_with("mod plan;\n"),
        "{parent}"
    );
    let plan = common::read_file(root, "src/engine/plan/mod.rs");
    assert!(plan.contains("pub(super) mod matching;"), "{plan}");
}
