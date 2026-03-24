use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn cli_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_refac"))
}

#[test]
fn markdown_move_keeps_external_uri_links_inside_moved_file() {
    let temp = tempdir().expect("failed to create temp dir");
    let project = temp.path();

    let source = project.join("target.md");
    let target = project.join("guides/target.md");

    fs::write(
        &source,
        "# Target\n\n[External](file:///tmp/guide.md)\n[Docs](https://example.com/guide)\n",
    )
    .expect("failed to write source markdown");

    let output = Command::new(cli_binary())
        .args([
            "move",
            "--project-path",
            project.to_str().expect("project path should be valid UTF-8"),
            "--source-path",
            source.to_str().expect("source path should be valid UTF-8"),
            "--target-path",
            target.to_str().expect("target path should be valid UTF-8"),
        ])
        .output()
        .expect("failed to execute CLI binary");

    assert!(
        output.status.success(),
        "markdown move should succeed:\nstdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let moved = fs::read_to_string(&target).expect("failed to read moved markdown file");
    assert!(
        moved.contains("(file:///tmp/guide.md)"),
        "file URI should stay unchanged:\n{moved}"
    );
    assert!(
        moved.contains("(https://example.com/guide)"),
        "https URI should stay unchanged:\n{moved}"
    );
}
