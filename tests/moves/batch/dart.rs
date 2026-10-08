//! A Dart batch updates the package: import between the two moved files.

use crate::common;
use crate::common::DART_LOCK;

#[test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
fn dart_batch_moves_two_files_and_updates_cross_import() {
    // validator.dart imports formatter.dart via package: URI.
    // Both move from lib/src/ to lib/src/core/ in one batch call.
    //
    // The Dart driver sends ALL renames in a single workspace/willRenameFiles
    // request, so the analysis server sees both moves atomically.
    //
    // After the move:
    //   validator.dart (at lib/src/core/) must import the new package URI
    //   package:acme_utils/src/core/formatter.dart
    let _lock = DART_LOCK.lock().unwrap();
    let temp = common::setup_fixture("dart/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--source-path",
        project.join("lib/src/validator.dart").to_str().unwrap(),
        "--target-path",
        project
            .join("lib/src/core/formatter.dart")
            .to_str()
            .unwrap(),
        "--target-path",
        project
            .join("lib/src/core/validator.dart")
            .to_str()
            .unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(
        project.join("lib/src/core/formatter.dart").exists(),
        "formatter.dart must be at target"
    );
    assert!(
        project.join("lib/src/core/validator.dart").exists(),
        "validator.dart must be at target"
    );
    assert!(
        !project.join("lib/src/formatter.dart").exists(),
        "formatter.dart must be gone from source"
    );
    assert!(
        !project.join("lib/src/validator.dart").exists(),
        "validator.dart must be gone from source"
    );

    // validator.dart's import of formatter.dart must point to the new package URI
    let validator = common::read_file(project, "lib/src/core/validator.dart");
    assert!(
        !validator.contains("package:acme_utils/src/formatter.dart"),
        "validator.dart: old formatter import must be gone:\n{validator}"
    );
    assert!(
        validator.contains("package:acme_utils/src/core/formatter.dart"),
        "validator.dart: import must be updated to the new package URI:\n{validator}"
    );
}
