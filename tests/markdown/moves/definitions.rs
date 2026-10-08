//! Reference-style links: the definitions follow the moved file, in other files and inside it.

use super::{assert_move_succeeded, contains_either, run_cli, write_file};

use std::fs;
use tempfile::tempdir;

#[test]
fn markdown_move_updates_reference_definitions_in_other_files() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(
        &project.join("overview.md"),
        "# Overview\n\nReview [the target][target].\n\n[target]: ./target.md#deep-dive \"Deep Dive\"\n",
    );
    write_file(
        &project.join("target.md"),
        "# Target\n\n## Deep Dive\n\nDetails.\n",
    );

    let source = project.join("target.md");
    let target = project.join("guides/target.md");
    let project_arg = project.to_str().unwrap();
    let source_arg = source.to_str().unwrap();
    let target_arg = target.to_str().unwrap();

    let output = run_cli(&[
        "move",
        "--project-path",
        project_arg,
        "--source-path",
        source_arg,
        "--target-path",
        target_arg,
    ]);

    assert_move_succeeded(&output);

    let overview =
        fs::read_to_string(project.join("overview.md")).expect("failed to read overview.md");
    assert!(
        !overview.contains("[target]: ./target.md#deep-dive"),
        "old reference definition should be gone:\n{overview}"
    );
    assert!(
        contains_either(
            &overview,
            "[target]: guides/target.md#deep-dive \"Deep Dive\"",
            "[target]: ./guides/target.md#deep-dive \"Deep Dive\""
        ),
        "rewritten reference definition missing or anchor/title was not preserved:\n{overview}"
    );
}

#[test]
fn markdown_move_recalculates_reference_definitions_inside_the_moved_file() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(&project.join("sibling.md"), "# Sibling\n");
    write_file(&project.join("nested/leaf.md"), "# Leaf\n\n## Details\n");
    write_file(
        &project.join("target.md"),
        "# Target\n\nSee [Sibling][sibling] and [Leaf][leaf].\n\n[sibling]: ./sibling.md\n[leaf]: ./nested/leaf.md#details \"Leaf Details\"\n",
    );

    let source = project.join("target.md");
    let target = project.join("guides/target.md");
    let project_arg = project.to_str().unwrap();
    let source_arg = source.to_str().unwrap();
    let target_arg = target.to_str().unwrap();

    let output = run_cli(&[
        "move",
        "--project-path",
        project_arg,
        "--source-path",
        source_arg,
        "--target-path",
        target_arg,
    ]);

    assert_move_succeeded(&output);

    let moved = fs::read_to_string(project.join("guides/target.md"))
        .expect("failed to read moved markdown file");
    assert!(
        !moved.contains("[sibling]: ./sibling.md"),
        "old sibling reference definition should be gone after move:\n{moved}"
    );
    assert!(
        !moved.contains("[leaf]: ./nested/leaf.md#details"),
        "old leaf reference definition should be gone after move:\n{moved}"
    );
    assert!(
        moved.contains("[sibling]: ../sibling.md"),
        "sibling reference definition was not recalculated relative to the new location:\n{moved}"
    );
    assert!(
        moved.contains("[leaf]: ../nested/leaf.md#details \"Leaf Details\""),
        "leaf reference definition was not recalculated relative to the new location:\n{moved}"
    );
}
