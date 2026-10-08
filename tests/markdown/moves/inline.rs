//! Inline links: other files follow the moved file, links inside it are recalculated, anchors survive.

use super::{assert_move_succeeded, contains_either, run_cli, write_file};

use std::fs;
use tempfile::tempdir;

#[test]
fn markdown_move_updates_other_files_that_reference_the_moved_file() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(
        &project.join("index.md"),
        "# Index\n\nSee [Target](./target.md).\n",
    );
    write_file(&project.join("target.md"), "# Target\n");

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

    let index = fs::read_to_string(project.join("index.md")).expect("failed to read index.md");
    assert!(
        !index.contains("./target.md"),
        "old inbound link should be gone:\n{index}"
    );
    assert!(
        contains_either(&index, "(guides/target.md)", "(./guides/target.md)"),
        "rewritten inbound link missing:\n{index}"
    );
}

#[test]
fn markdown_move_recalculates_relative_links_inside_the_moved_file() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(&project.join("sibling.md"), "# Sibling\n");
    write_file(&project.join("nested/leaf.md"), "# Leaf\n");
    write_file(
        &project.join("target.md"),
        "# Target\n\n[Sibling](./sibling.md)\n[Leaf](./nested/leaf.md)\n",
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
        !moved.contains("(./sibling.md)"),
        "old sibling link should be gone after move:\n{moved}"
    );
    assert!(
        !moved.contains("(./nested/leaf.md)"),
        "old nested link should be gone after move:\n{moved}"
    );
    assert!(
        moved.contains("(../sibling.md)"),
        "sibling link was not recalculated relative to the new location:\n{moved}"
    );
    assert!(
        moved.contains("(../nested/leaf.md)"),
        "nested link was not recalculated relative to the new location:\n{moved}"
    );
}

#[test]
fn markdown_move_preserves_anchor_fragments_when_rewriting_references() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(
        &project.join("overview.md"),
        "# Overview\n\nJump to [Target Section](./target.md#deep-dive).\n",
    );
    write_file(
        &project.join("target.md"),
        "# Target\n\n## Deep Dive\n\nDetails.\n",
    );

    let source = project.join("target.md");
    let target = project.join("reference/target.md");
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
        !overview.contains("./target.md#deep-dive"),
        "old anchored link should be gone:\n{overview}"
    );
    assert!(
        contains_either(
            &overview,
            "(reference/target.md#deep-dive)",
            "(./reference/target.md#deep-dive)"
        ),
        "rewritten anchored link missing or anchor was not preserved:\n{overview}"
    );
}
