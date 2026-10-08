//! Imports and exports that follow the moved file: package: and relative URIs, aliases, show combinators, barrel files.

use super::run_move;
use crate::common;
use crate::common::DART_LOCK;

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_places_file_at_target_and_removes_source() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
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

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_updates_barrel_export() {
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
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_updates_package_import() {
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
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_preserves_show_combinator_on_package_import() {
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
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_updates_relative_import_and_preserves_alias() {
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
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_preserves_show_combinator_on_item_import() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
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

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_updates_relative_import_and_preserves_alias_in_api_client() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
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

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_rewrites_all_package_imports() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
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

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_move_rewrites_all_relative_imports() {
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();
    let _lock = DART_LOCK.lock().unwrap();
    common::assert_move_succeeded(&run_move(project));

    // Files in lib/src/ (same level as formatter.dart): 'formatter.dart' → 'core/formatter.dart'
    let same_level = [(
        "lib/src/service.dart",
        "'formatter.dart'",
        "'core/formatter.dart'",
    )];
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
