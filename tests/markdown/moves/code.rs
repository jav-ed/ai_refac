//! Code is not Markdown: links in fenced blocks and inline code spans stay as written.

use super::{assert_move_succeeded, contains_either, run_cli, write_file};

use std::fs;
use tempfile::tempdir;

#[test]
fn markdown_move_does_not_rewrite_links_inside_fenced_code_blocks() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(
        &project.join("index.md"),
        "# Index\n\nReal link: [Target](./target.md)\n\n```md\nExample: [Target](./target.md)\n```\n",
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
        contains_either(&index, "(guides/target.md)", "(./guides/target.md)"),
        "real link outside code block should be updated:\n{index}"
    );
    assert!(
        index.contains("```md\nExample: [Target](./target.md)\n```"),
        "link inside fenced code block should not be rewritten:\n{index}"
    );
}

#[test]
fn markdown_move_does_not_rewrite_links_inside_inline_code_spans() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    write_file(
        &project.join("index.md"),
        "# Index\n\nReal: [Target](./target.md). Example: `[Target](./target.md)`.\n",
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
        contains_either(&index, "(guides/target.md)", "(./guides/target.md)"),
        "real link outside inline code should be updated:\n{index}"
    );
    assert!(
        index.contains("`[Target](./target.md)`"),
        "link inside inline code span should not be rewritten:\n{index}"
    );
}
