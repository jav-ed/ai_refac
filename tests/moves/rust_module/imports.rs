//! Imports of the moved module: short references stay short, and a module leaves a group it can no longer be named from.

use super::{run, write};
use crate::common;

/// A reference written `name::item` in a file that imports the module keeps its short form: the
/// import is rewritten and brings the new name, so the body is renamed, not made absolute.
#[test]
fn short_references_through_an_import_stay_short() {
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
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );
    write(
        root,
        "src/consumer.rs",
        "use crate::engine::matching;\n\npub fn consume() -> u32 { matching::value() + matching::value() }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::scoring");
    common::assert_move_succeeded(&output);

    assert_eq!(
        common::read_file(root, "src/consumer.rs"),
        "use crate::domain::scoring;\n\npub fn consume() -> u32 { scoring::value() + scoring::value() }\n"
    );
}

#[test]
fn a_module_leaves_the_group_of_an_import_that_can_no_longer_reach_it() {
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
        "pub mod helper;\npub mod matching;\nmod user;\n",
    );
    write(root, "src/engine/helper.rs", "pub fn help() -> u32 { 1 }\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );
    write(
        root,
        "src/engine/user.rs",
        "use super::{helper, matching};\n\npub fn go() -> u32 { helper::help() + matching::value() }\n",
    );

    let output = run(
        root,
        "crate::engine::matching",
        "crate::engine::plan::matching",
    );
    common::assert_move_succeeded(&output);

    let user = common::read_file(root, "src/engine/user.rs");
    assert!(
        user.starts_with("use super::{helper};\nuse crate::engine::plan::matching;\n"),
        "{user}"
    );
}
