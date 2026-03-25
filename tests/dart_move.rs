mod common;

// Fixture: tests/fixtures/dart/project/  (20 files)
//
// Move under test: lib/src/formatter.dart  ->  lib/src/core/formatter.dart
//
// The driver uses the Dart analysis server via workspace/willRenameFiles LSP.
// The server handles all URI rewriting — both package: and relative imports,
// plus export directives in barrel files.
//
// Patterns exercised:
//   - Barrel export directive (lib/acme_utils.dart) — export URI updated
//   - package: import — updated when imported file moves (validator.dart, order.dart, etc.)
//   - package: import with show combinator — URI updates, combinator survives (item.dart)
//   - Relative import from same directory (service.dart): 'formatter.dart' → 'core/formatter.dart'
//   - Relative import from subdirectory (models/, utils/, network/, cache/): '../formatter.dart'
//   - Relative import + as alias (service.dart as fmt, api_client.dart as f)
//   - dart: SDK imports must NOT be rewritten (dart:io, dart:convert)
//   - Control files with no formatter dep: byte-identical after move

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project.join("lib/src/core/formatter.dart").to_str().unwrap(),
    ])
}

// ── file placement ─────────────────────────────────────────────────────────

#[test]
fn dart_move_places_file_at_target_and_removes_source() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    assert!(
        project.join("lib/src/core/formatter.dart").exists(),
        "formatter.dart must exist at target path"
    );
    assert!(
        !project.join("lib/src/formatter.dart").exists(),
        "formatter.dart must be gone from source path"
    );
}

// ── existing tests (inlined CLI calls kept as-is) ──────────────────────────

#[test]
fn dart_move_updates_barrel_export() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project.join("lib/src/core/formatter.dart").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    let barrel = common::read_file(project, "lib/acme_utils.dart");
    assert!(
        !barrel.contains("'src/formatter.dart'"),
        "old export path should be gone from barrel:\n{barrel}"
    );
    assert!(
        barrel.contains("'src/core/formatter.dart'"),
        "updated export path missing in barrel:\n{barrel}"
    );
}

#[test]
fn dart_move_updates_package_import() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project.join("lib/src/core/formatter.dart").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // validator.dart uses `import 'package:acme_utils/src/formatter.dart'`
    let validator = common::read_file(project, "lib/src/validator.dart");
    assert!(
        !validator.contains("'package:acme_utils/src/formatter.dart'"),
        "old package: import should be gone from validator.dart:\n{validator}"
    );
    assert!(
        validator.contains("'package:acme_utils/src/core/formatter.dart'"),
        "updated package: import missing in validator.dart:\n{validator}"
    );
}

#[test]
fn dart_move_preserves_show_combinator_on_package_import() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project.join("lib/src/core/formatter.dart").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    let test_file = common::read_file(project, "test/formatter_test.dart");
    assert!(
        !test_file.contains("'package:acme_utils/src/formatter.dart'"),
        "old package: import should be gone from formatter_test.dart:\n{test_file}"
    );
    assert!(
        test_file.contains("'package:acme_utils/src/core/formatter.dart'"),
        "updated package: import missing in formatter_test.dart:\n{test_file}"
    );
    // The show combinator must survive verbatim
    assert!(
        test_file.contains("show Formatter"),
        "'show Formatter' combinator should be preserved:\n{test_file}"
    );
}

#[test]
fn dart_move_updates_relative_import_and_preserves_alias() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project.join("lib/src/core/formatter.dart").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // service.dart uses `import 'formatter.dart' as fmt`
    let service = common::read_file(project, "lib/src/service.dart");
    assert!(
        !service.contains("'formatter.dart'"),
        "old relative import should be gone from service.dart:\n{service}"
    );
    assert!(
        service.contains("'core/formatter.dart'"),
        "updated relative import missing in service.dart:\n{service}"
    );
    // Alias must survive
    assert!(
        service.contains("as fmt"),
        "alias 'fmt' should be preserved:\n{service}"
    );
}

#[test]
fn dart_move_does_not_rewrite_dart_sdk_imports() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project.join("lib/src/core/formatter.dart").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // The moved file itself uses `import 'dart:convert'` — must not be touched.
    let moved = common::read_file(project, "lib/src/core/formatter.dart");
    assert!(
        moved.contains("import 'dart:convert'"),
        "dart: SDK import should be preserved in moved file:\n{moved}"
    );
}

// ── dart:io in a separate file is also untouched ───────────────────────────

#[test]
fn dart_move_does_not_rewrite_dart_io_import_in_service() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let service = common::read_file(project, "lib/src/service.dart");
    assert!(
        service.contains("import 'dart:io'"),
        "dart:io import in service.dart must be preserved:\n{service}"
    );
}

// ── show combinator on item.dart (package: + show) ─────────────────────────

#[test]
fn dart_move_preserves_show_combinator_on_item_import() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let item = common::read_file(project, "lib/src/models/item.dart");
    assert!(
        item.contains("'package:acme_utils/src/core/formatter.dart'"),
        "item.dart: package: import must be updated to src/core/:\n{item}"
    );
    assert!(
        item.contains("show Formatter"),
        "item.dart: show combinator must survive the URI rewrite:\n{item}"
    );
}

// ── as alias on api_client.dart (relative + as) ────────────────────────────

#[test]
fn dart_move_updates_relative_import_and_preserves_alias_in_api_client() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let api = common::read_file(project, "lib/src/network/api_client.dart");
    assert!(
        !api.contains("'../formatter.dart'"),
        "api_client.dart: old relative import must be gone:\n{api}"
    );
    assert!(
        api.contains("'../core/formatter.dart'"),
        "api_client.dart: updated relative import missing:\n{api}"
    );
    assert!(
        api.contains("as f"),
        "api_client.dart: alias `as f` must be preserved:\n{api}"
    );
}

// ── bulk: all package: imports updated ────────────────────────────────────

#[test]
fn dart_move_rewrites_all_package_imports() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let files = [
        "lib/src/validator.dart",
        "lib/src/models/order.dart",
        "lib/src/models/item.dart",
        "lib/src/network/http_client.dart",
        "lib/src/analytics/tracker.dart",
        "test/formatter_test.dart",
        "test/validator_test.dart",
        "test/service_test.dart",
    ];

    for rel in &files {
        let content = common::read_file(project, rel);
        assert!(
            !content.contains("'package:acme_utils/src/formatter.dart'"),
            "{rel}: old package: URI must be gone:\n{content}"
        );
        assert!(
            content.contains("'package:acme_utils/src/core/formatter.dart'"),
            "{rel}: updated package: URI missing:\n{content}"
        );
    }
}

// ── bulk: all relative imports updated ────────────────────────────────────

#[test]
fn dart_move_rewrites_all_relative_imports() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // Files in lib/src/ (same level as formatter.dart): 'formatter.dart' → 'core/formatter.dart'
    let same_level = [
        ("lib/src/service.dart", "'formatter.dart'", "'core/formatter.dart'"),
    ];
    for (rel, old, new) in &same_level {
        let content = common::read_file(project, rel);
        assert!(
            !content.contains(old),
            "{rel}: old relative import {old} must be gone:\n{content}"
        );
        assert!(
            content.contains(new),
            "{rel}: updated relative import {new} missing:\n{content}"
        );
    }

    // Files in lib/src/<subdir>/: '../formatter.dart' → '../core/formatter.dart'
    let subdir_files = [
        "lib/src/models/user.dart",
        "lib/src/utils/string_utils.dart",
        "lib/src/utils/date_utils.dart",
        "lib/src/network/api_client.dart",
        "lib/src/cache/cache.dart",
    ];
    for rel in &subdir_files {
        let content = common::read_file(project, rel);
        assert!(
            !content.contains("'../formatter.dart'"),
            "{rel}: old relative import '../formatter.dart' must be gone:\n{content}"
        );
        assert!(
            content.contains("'../core/formatter.dart'"),
            "{rel}: updated relative import '../core/formatter.dart' missing:\n{content}"
        );
    }
}

// ── control files: no formatter dep, byte-identical ───────────────────────

#[test]
fn dart_move_leaves_control_files_unchanged() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

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

// ── http_client.dart: dart:io preserved alongside updated package: import ──

#[test]
fn dart_move_preserves_dart_io_in_http_client_while_updating_package_import() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
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
