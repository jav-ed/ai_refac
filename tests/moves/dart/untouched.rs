//! What the move must leave alone: dart: SDK imports and files with no dependency on the moved file.

use super::run_move;
use crate::common;
use crate::common::DART_LOCK;

#[test]
fn dart_move_does_not_rewrite_dart_sdk_imports() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();

    let output = common::run_cli(&[
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
    ]);

    common::assert_move_succeeded(&output);

    // The moved file itself uses `import 'dart:convert'` — must not be touched.
    let moved = common::read_file(project, "lib/src/core/formatter.dart");
    assert!(
        moved.contains("import 'dart:convert'"),
        "dart: SDK import should be preserved in moved file:\n{moved}"
    );
}

#[test]
fn dart_move_does_not_rewrite_dart_io_import_in_service() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
    common::assert_move_succeeded(&run_move(project));

    let service = common::read_file(project, "lib/src/service.dart");
    assert!(
        service.contains("import 'dart:io'"),
        "dart:io import in service.dart must be preserved:\n{service}"
    );
}

#[test]
fn dart_move_leaves_control_files_unchanged() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();

    let control_files = [
        "lib/src/config.dart",
        "lib/src/models/index.dart",
        "lib/src/utils/index.dart",
    ];
    let snapshots: Vec<(&str, String)> = control_files
        .iter()
        .map(|p| (*p, common::read_file(project, p)))
        .collect();

    common::assert_move_succeeded(&run_move(project));

    for (rel, before) in &snapshots {
        let after = common::read_file(project, rel);
        assert_eq!(
            after, *before,
            "{rel} has no formatter dependency — must be byte-identical after move"
        );
    }
}

#[test]
fn dart_move_preserves_dart_io_in_http_client_while_updating_package_import() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
    common::assert_move_succeeded(&run_move(project));

    let http = common::read_file(project, "lib/src/network/http_client.dart");
    assert!(
        http.contains("import 'dart:io'"),
        "http_client.dart: dart:io must be preserved:\n{http}"
    );
    assert!(
        http.contains("'package:acme_utils/src/core/formatter.dart'"),
        "http_client.dart: package: import must be updated:\n{http}"
    );
    assert!(
        !http.contains("'package:acme_utils/src/formatter.dart'"),
        "http_client.dart: old package: import must be gone:\n{http}"
    );
}
