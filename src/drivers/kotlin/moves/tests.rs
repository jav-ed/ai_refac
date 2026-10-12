use super::*;
use crate::drivers::kotlin::android::moved::{locate, relocate};

fn step(from: &str, to: &str) -> Step {
    Step {
        from: PathBuf::from(from),
        to: PathBuf::from(to),
        is_dir: false,
    }
}

#[test]
fn an_edit_under_the_destination_is_redirected_to_the_source() {
    let steps = [step("/missing/old/A.kt", "/missing/new/A.kt")];
    assert_eq!(
        locate(Path::new("/missing/new/A.kt"), &steps),
        PathBuf::from("/missing/old/A.kt")
    );
    // An unrelated path is left alone, and fails loudly when it is read.
    assert_eq!(
        locate(Path::new("/missing/other/B.kt"), &steps),
        PathBuf::from("/missing/other/B.kt")
    );
}

#[test]
fn files_inside_a_moved_directory_are_relocated() {
    let steps = [step("/p/util", "/p/common")];
    assert_eq!(
        relocate(Path::new("/p/util/deep/A.kt"), &steps),
        PathBuf::from("/p/common/deep/A.kt")
    );
    assert_eq!(
        relocate(Path::new("/p/app/B.kt"), &steps),
        PathBuf::from("/p/app/B.kt")
    );
}

#[test]
fn a_directory_step_lists_its_source_files_with_their_new_place() {
    let dir = tempfile::tempdir().unwrap();
    let util = dir.path().join("util");
    std::fs::create_dir_all(util.join("deep")).unwrap();
    std::fs::write(util.join("A.kt"), "").unwrap();
    std::fs::write(util.join("deep/B.java"), "").unwrap();
    std::fs::write(util.join("notes.txt"), "").unwrap();
    let step = Step {
        from: util.clone(),
        to: dir.path().join("common"),
        is_dir: true,
    };

    let mut files = source_files(&step).unwrap();
    files.sort();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].1, dir.path().join("common/A.kt"));
    assert_eq!(files[1].1, dir.path().join("common/deep/B.java"));
}

/// A folder that changes its parent and its name is two steps, and the second
/// starts at a path the first one creates. The check before the server starts
/// failed on that path ("IO error ... No such file") in every Kotlin project.
#[test]
fn a_folder_moved_and_renamed_passes_the_check_before_the_server_starts() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let kotlin = root.join("src/main/kotlin/com/x");
    std::fs::create_dir_all(kotlin.join("core/personal")).unwrap();
    std::fs::create_dir_all(kotlin.join("feature/personal")).unwrap();
    std::fs::write(
        kotlin.join("core/personal/Notes.kt"),
        "package com.x.core.personal\n",
    )
    .unwrap();
    std::fs::write(root.join("settings.gradle.kts"), "").unwrap();

    let plan = plan::build(
        &[(
            kotlin.join("core/personal").display().to_string(),
            kotlin.join("feature/personal/data").display().to_string(),
        )],
        root,
    )
    .unwrap();

    assert_eq!(plan.groups.len(), 2, "move, then rename");
    refuse_declared_pairs(&plan, root).unwrap();
}
