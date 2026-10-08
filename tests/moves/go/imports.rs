//! Imports and call sites that follow the moved file: aliased, unaliased, plain, in every file that uses the package.

use super::run_move;
use crate::common;

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
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

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
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
#[ignore = "needs gopls (run `refac doctor go`)"]
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
#[ignore = "needs gopls (run `refac doctor go`)"]
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

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
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

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
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
#[ignore = "needs gopls (run `refac doctor go`)"]
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

#[test]
#[ignore = "needs gopls (run `refac doctor go`)"]
fn go_move_updates_validate_only_import_to_pkg_helpers() {
    let temp = common::setup_fixture("go/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // gopls renames the whole package — validate.go moves to pkg/helpers/ too.
    // Callers that only use Validate() also get their import rewritten.
    let validator = common::read_file(project, "internal/validator/validator.go");
    assert!(
        validator.contains("\"github.com/example/myproject/pkg/helpers\""),
        "internal/validator: import must be pkg/helpers (whole package renamed):\n{validator}"
    );
    assert!(
        !validator.contains("pkg/utils\""),
        "internal/validator: old pkg/utils import must be gone:\n{validator}"
    );
    assert!(
        validator.contains("helpers.Validate"),
        "internal/validator: call site must be helpers.Validate:\n{validator}"
    );
}
