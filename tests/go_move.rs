mod common;

// Fixture: tests/fixtures/go/project/  (21 files)
//
// Move under test: pkg/utils/format.go  ->  pkg/helpers/format.go
//
// Go-specific: moving a file changes its package (directory = package).
// The driver uses gopls via textDocument/rename on the package name symbol.
// gopls cascades the rename to:
//   1. Package declaration in the moved file (package utils → package helpers)
//   2. All import path strings referencing pkg/utils for symbols in format.go
//   3. All unaliased call-site qualifiers (utils.X → helpers.X)
//
// Aliased imports keep the alias — only the path string changes.
//   u "pkg/utils" → u "pkg/helpers"   (qualifier u.X is unchanged)
//
// Blank imports of unrelated packages (_ "pkg/setup") are untouched.
//
// Partial-package-move edge case: pkg/utils/validate.go stays in pkg/utils/.
// The driver renames based on the moved file's new directory, not the whole
// source package. validate.go keeps `package utils` and callers that only
// use utils.Validate() keep their pkg/utils import.

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ])
}

// ── file placement ─────────────────────────────────────────────────────────

#[test]
fn go_move_places_file_at_target_and_removes_source() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    assert!(
        project.join("pkg/helpers/format.go").exists(),
        "format.go must exist at target path"
    );
    assert!(
        !project.join("pkg/utils/format.go").exists(),
        "format.go must be gone from source path"
    );
}

// ── partial-package-move: validate.go stays ────────────────────────────────

#[test]
fn go_move_leaves_validate_go_in_place() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    let before = common::read_file(project, "pkg/utils/validate.go");
    common::assert_move_succeeded(&run_move(project));

    assert!(
        project.join("pkg/utils/validate.go").exists(),
        "validate.go must stay in pkg/utils/ — only format.go moves"
    );
    let after = common::read_file(project, "pkg/utils/validate.go");
    assert_eq!(
        after, before,
        "validate.go must be byte-identical after move (it does not move)"
    );
}

// ── existing tests (inlined CLI calls kept as-is) ──────────────────────────

#[test]
fn go_move_updates_aliased_import_in_main() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    let main = common::read_file(project, "cmd/main.go");
    assert!(
        !main.contains("\"github.com/example/myproject/pkg/utils\""),
        "old package path should be gone from main.go:\n{main}"
    );
    assert!(
        main.contains("\"github.com/example/myproject/pkg/helpers\""),
        "updated package path missing in main.go:\n{main}"
    );
    // Alias must survive
    assert!(
        main.contains("u \"github.com/example/myproject/pkg/helpers\""),
        "aliased import should preserve alias 'u':\n{main}"
    );
}

#[test]
fn go_move_updates_unaliased_import_and_call_sites() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    let service = common::read_file(project, "internal/service/service.go");
    assert!(
        !service.contains("\"github.com/example/myproject/pkg/utils\""),
        "old package path should be gone from service.go:\n{service}"
    );
    assert!(
        service.contains("\"github.com/example/myproject/pkg/helpers\""),
        "updated package path missing in service.go:\n{service}"
    );
    // Call sites: qualifier must change from utils. to helpers.
    assert!(
        !service.contains("utils.FormatValue") && !service.contains("utils.IsValid"),
        "old package qualifier should be gone from call sites:\n{service}"
    );
    assert!(
        service.contains("helpers.FormatValue") && service.contains("helpers.IsValid"),
        "updated package qualifier missing at call sites:\n{service}"
    );
}

#[test]
fn go_move_does_not_touch_blank_import_of_unrelated_package() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // The blank import of pkg/setup is unrelated — it must not be touched.
    let main = common::read_file(project, "cmd/main.go");
    assert!(
        main.contains("_ \"github.com/example/myproject/pkg/setup\""),
        "blank import of unrelated package should be preserved:\n{main}"
    );
}

#[test]
fn go_move_updates_package_declaration_in_moved_file() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    let moved = common::read_file(project, "pkg/helpers/format.go");
    assert!(
        !moved.contains("package utils"),
        "old package declaration should be gone:\n{moved}"
    );
    assert!(
        moved.contains("package helpers"),
        "new package declaration missing:\n{moved}"
    );
}

// ── second aliased import (cmd/server/main.go) ─────────────────────────────

#[test]
fn go_move_updates_second_aliased_import_and_preserves_its_alias() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // cmd/server/main.go uses alias `f` instead of `u`
    let server = common::read_file(project, "cmd/server/main.go");
    assert!(
        server.contains("f \"github.com/example/myproject/pkg/helpers\""),
        "cmd/server/main.go: alias `f` must survive; path must become pkg/helpers:\n{server}"
    );
    assert!(
        !server.contains("pkg/utils\""),
        "cmd/server/main.go: old pkg/utils import must be gone:\n{server}"
    );
    // Aliased qualifier is unchanged (f.FormatValue, not helpers.FormatValue)
    assert!(
        server.contains("f.FormatValue"),
        "cmd/server/main.go: call site `f.FormatValue` must be unchanged (alias preserved):\n{server}"
    );
}

// ── plain import in cmd/worker ─────────────────────────────────────────────

#[test]
fn go_move_rewrites_unaliased_import_and_call_sites_in_worker() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let worker = common::read_file(project, "cmd/worker/main.go");
    assert!(
        worker.contains("\"github.com/example/myproject/pkg/helpers\""),
        "cmd/worker: import must be pkg/helpers:\n{worker}"
    );
    assert!(
        !worker.contains("pkg/utils\""),
        "cmd/worker: old pkg/utils import must be gone:\n{worker}"
    );
    assert!(
        worker.contains("helpers.FormatValue"),
        "cmd/worker: call site must be helpers.FormatValue:\n{worker}"
    );
    assert!(
        worker.contains("helpers.IsValid"),
        "cmd/worker: call site must be helpers.IsValid:\n{worker}"
    );
}

// ── bulk: all plain-import files updated ───────────────────────────────────

#[test]
fn go_move_rewrites_all_plain_import_files() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let files = [
        "cmd/worker/main.go",
        "internal/service/service.go",
        "internal/auth/auth.go",
        "internal/repo/repo.go",
        "pkg/models/user.go",
        "pkg/models/order.go",
        "pkg/models/item.go",
        "pkg/services/user_service.go",
        "pkg/services/order_service.go",
        "pkg/api/router.go",
        "pkg/api/handlers.go",
        "pkg/api/middleware.go",
        "pkg/reports/report.go",
    ];

    for rel in &files {
        let content = common::read_file(project, rel);
        assert!(
            content.contains("\"github.com/example/myproject/pkg/helpers\""),
            "{rel}: import must be pkg/helpers after move:\n{content}"
        );
        assert!(
            !content.contains("pkg/utils\""),
            "{rel}: old pkg/utils import must be gone:\n{content}"
        );
    }
}

#[test]
fn go_move_rewrites_all_call_sites_to_helpers_qualifier() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let files = [
        "cmd/worker/main.go",
        "internal/service/service.go",
        "internal/auth/auth.go",
        "internal/repo/repo.go",
        "pkg/models/user.go",
        "pkg/models/order.go",
        "pkg/models/item.go",
        "pkg/services/user_service.go",
        "pkg/services/order_service.go",
        "pkg/api/router.go",
        "pkg/api/handlers.go",
        "pkg/api/middleware.go",
        "pkg/reports/report.go",
    ];

    for rel in &files {
        let content = common::read_file(project, rel);
        assert!(
            !content.contains("utils.FormatValue") && !content.contains("utils.IsValid"),
            "{rel}: old utils.X call sites must be gone:\n{content}"
        );
    }
}

// ── validate.go-only caller: import stays pkg/utils ────────────────────────

#[test]
fn go_move_leaves_validate_only_import_as_pkg_utils() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    let before = common::read_file(project, "internal/validator/validator.go");
    common::assert_move_succeeded(&run_move(project));

    // validator.go only calls utils.Validate() which lives in validate.go — not the moved file.
    // gopls knows which symbols come from which file; callers of validate.go keep pkg/utils.
    let after = common::read_file(project, "internal/validator/validator.go");
    assert_eq!(
        after, before,
        "validator.go must be byte-identical: it only uses Validate from validate.go (stays in pkg/utils)"
    );
}

// ── control file ───────────────────────────────────────────────────────────

#[test]
fn go_move_leaves_control_file_unchanged() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    let before = common::read_file(project, "config/config.go");
    common::assert_move_succeeded(&run_move(project));

    let after = common::read_file(project, "config/config.go");
    assert_eq!(
        after, before,
        "config/config.go has no pkg/utils dependency — must be byte-identical after move"
    );
}

#[test]
fn go_move_leaves_setup_package_unchanged() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    let before = common::read_file(project, "pkg/setup/init.go");
    common::assert_move_succeeded(&run_move(project));

    let after = common::read_file(project, "pkg/setup/init.go");
    assert_eq!(
        after, before,
        "pkg/setup/init.go is the blank-import target — must be byte-identical after move"
    );
}
