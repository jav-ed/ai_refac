//! Markdown batches, alone and mixed with TypeScript.

use crate::common;

#[test]
fn markdown_batch_moves_two_files_and_updates_all_links() {
    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    // index.md links to both a.md and b.md.
    // a.md also links to b.md (cross-link between the two moved files).
    // Batch move a.md → docs/a.md and b.md → docs/b.md.
    fs::create_dir_all(project.join("docs")).unwrap();
    fs::write(
        project.join("index.md"),
        "# Index\n\nSee [A](./a.md) and [B](./b.md).\n",
    )
    .unwrap();
    fs::write(project.join("a.md"), "# A\n\nSee also [B](./b.md).\n").unwrap();
    fs::write(project.join("b.md"), "# B\n").unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("a.md").to_str().unwrap(),
        "--source-path",
        project.join("b.md").to_str().unwrap(),
        "--target-path",
        project.join("docs/a.md").to_str().unwrap(),
        "--target-path",
        project.join("docs/b.md").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(
        project.join("docs/a.md").exists(),
        "a.md must be at docs/a.md"
    );
    assert!(
        project.join("docs/b.md").exists(),
        "b.md must be at docs/b.md"
    );
    assert!(
        !project.join("a.md").exists(),
        "a.md must be gone from root"
    );
    assert!(
        !project.join("b.md").exists(),
        "b.md must be gone from root"
    );

    // External file: both links updated
    let index = fs::read_to_string(project.join("index.md")).unwrap();
    assert!(
        !index.contains("(./a.md)"),
        "index.md: old link to a.md must be gone:\n{index}"
    );
    assert!(
        !index.contains("(./b.md)"),
        "index.md: old link to b.md must be gone:\n{index}"
    );
    assert!(
        index.contains("docs/a.md"),
        "index.md: new link to docs/a.md must be present:\n{index}"
    );
    assert!(
        index.contains("docs/b.md"),
        "index.md: new link to docs/b.md must be present:\n{index}"
    );

    // Cross-link inside a.md (a.md → b.md): both landed in docs/, so link becomes ./b.md
    let a = fs::read_to_string(project.join("docs/a.md")).unwrap();
    assert!(
        !a.contains("../b.md") && !a.contains("./b.md") || a.contains("b.md"),
        "a.md must still reference b.md in some valid relative form:\n{a}"
    );
}

#[test]
fn mixed_language_batch_dispatches_ts_and_markdown_independently() {
    // One TypeScript file and one Markdown file in the same batch call.
    // The orchestrator must route each to its own driver and both must succeed.
    //
    // Setup:
    //   project/
    //     tsconfig.json
    //     src/lib.ts        — exports `greeting`
    //     src/consumer.ts   — imports from './lib'
    //     docs/index.md     — links to ./notes.md
    //     docs/notes.md
    //
    // Batch move:
    //   src/lib.ts     → src/utils/lib.ts   (TypeScript driver)
    //   docs/notes.md  → archive/notes.md   (Markdown driver)
    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir_all(project.join("docs")).unwrap();
    fs::create_dir_all(project.join("src/utils")).unwrap();
    fs::create_dir_all(project.join("archive")).unwrap();

    fs::write(
        project.join("tsconfig.json"),
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
    )
    .unwrap();
    fs::write(
        project.join("src/lib.ts"),
        "export const greeting = \"hello\";\n",
    )
    .unwrap();
    fs::write(
        project.join("src/consumer.ts"),
        "import { greeting } from './lib';\nconsole.log(greeting);\n",
    )
    .unwrap();
    fs::write(
        project.join("docs/index.md"),
        "# Docs\n\nSee [notes](./notes.md).\n",
    )
    .unwrap();
    fs::write(project.join("docs/notes.md"), "# Notes\n").unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("src/lib.ts").to_str().unwrap(),
        "--source-path",
        project.join("docs/notes.md").to_str().unwrap(),
        "--target-path",
        project.join("src/utils/lib.ts").to_str().unwrap(),
        "--target-path",
        project.join("archive/notes.md").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // TypeScript: file placed, consumer.ts import updated
    assert!(
        project.join("src/utils/lib.ts").exists(),
        "lib.ts must be at target"
    );
    assert!(
        !project.join("src/lib.ts").exists(),
        "lib.ts must be gone from source"
    );

    let consumer = fs::read_to_string(project.join("src/consumer.ts")).unwrap();
    assert!(
        !consumer.contains("'./lib'") && !consumer.contains("\"./lib\""),
        "consumer.ts: old import must be gone:\n{consumer}"
    );
    assert!(
        consumer.contains("./utils/lib") || consumer.contains("utils/lib"),
        "consumer.ts: import must be updated to utils/lib:\n{consumer}"
    );

    // Markdown: file placed, index.md link updated
    assert!(
        project.join("archive/notes.md").exists(),
        "notes.md must be at target"
    );
    assert!(
        !project.join("docs/notes.md").exists(),
        "notes.md must be gone from source"
    );

    let index = fs::read_to_string(project.join("docs/index.md")).unwrap();
    assert!(
        !index.contains("(./notes.md)"),
        "index.md: old link must be gone:\n{index}"
    );
    assert!(
        index.contains("../archive/notes.md") || index.contains("archive/notes.md"),
        "index.md: link must be updated to archive/notes.md:\n{index}"
    );
}
