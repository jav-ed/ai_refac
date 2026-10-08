use crate::common;

// The Dart server rewrites `package:` imports only when
// `.dart_tool/package_config.json` exists. Without it a move would leave
// dangling imports behind, so the driver has to refuse before writing anything.
//
// Fixture: tests/fixtures/dart/project/, the move of dart_move.rs
// (lib/src/formatter.dart -> lib/src/core/formatter.dart). With the config file
// in place that move is covered there. This is its own test binary with a single
// test, so it never runs next to another Dart server and needs no lock.

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project
            .join("lib/src/core/formatter.dart")
            .to_str()
            .unwrap(),
    ])
}

/// Every file below `root` with its bytes, in path order.
fn snapshot(root: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut files: Vec<_> = walkdir::WalkDir::new(root)
        .into_iter()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            let bytes = std::fs::read(entry.path()).unwrap();
            (entry.path().to_path_buf(), bytes)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn dart_move_without_package_config_refuses_and_changes_nothing() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    std::fs::remove_file(project.join(".dart_tool/package_config.json")).unwrap();
    let before = snapshot(project);

    let output = run_move(project);

    assert!(
        !output.status.success(),
        "a move that would leave imports dangling must fail"
    );
    let stderr = common::stderr_text(&output);
    assert!(
        stderr.contains("package_config.json") && stderr.contains("Nothing was changed"),
        "the error must name the missing file and say nothing was changed:\n{stderr}"
    );
    assert_eq!(
        snapshot(project),
        before,
        "a refused move must leave every file as it was"
    );
}
