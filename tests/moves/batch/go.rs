//! Go batches. The same-package batch is the high-risk case: gopls moves the whole package on the first call, so a naive second call would fail.

use crate::common;

#[test]
fn go_batch_same_package_moves_both_files_and_updates_all_callers() {
    // pkg/utils/ contains format.go AND validate.go (same package).
    // Batch-moving both triggers the deduplication fix: only one gopls rename
    // is issued (gopls moves the whole package on the first call).
    //
    // Verifies: file placement, package declarations, ALL caller import paths
    // and call-site qualifiers — same coverage as the single-file go_move tests.
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/validate.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/validate.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // File placement
    assert!(
        project.join("pkg/helpers/format.go").exists(),
        "format.go must be at target"
    );
    assert!(
        project.join("pkg/helpers/validate.go").exists(),
        "validate.go must be at target"
    );
    assert!(
        !project.join("pkg/utils/format.go").exists(),
        "format.go must be gone from source"
    );
    assert!(
        !project.join("pkg/utils/validate.go").exists(),
        "validate.go must be gone from source"
    );

    // Package declarations updated
    let fmt = common::read_file(project, "pkg/helpers/format.go");
    assert!(
        fmt.contains("package helpers"),
        "format.go: package must be helpers:\n{fmt}"
    );
    assert!(
        !fmt.contains("package utils"),
        "format.go: old package must be gone:\n{fmt}"
    );

    let val = common::read_file(project, "pkg/helpers/validate.go");
    assert!(
        val.contains("package helpers"),
        "validate.go: package must be helpers:\n{val}"
    );

    // All plain-import callers must have import path and call-site qualifiers updated
    let plain_callers = [
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
    for rel in &plain_callers {
        let content = common::read_file(project, rel);
        assert!(
            content.contains("\"github.com/example/myproject/pkg/helpers\""),
            "{rel}: import must be pkg/helpers:\n{content}"
        );
        assert!(
            !content.contains("pkg/utils\""),
            "{rel}: old pkg/utils import must be gone:\n{content}"
        );
        assert!(
            !content.contains("utils.FormatValue") && !content.contains("utils.IsValid"),
            "{rel}: old utils.X call sites must be gone:\n{content}"
        );
    }

    // Aliased import in cmd/main.go — alias survives, path updated
    let main = common::read_file(project, "cmd/main.go");
    assert!(
        main.contains("u \"github.com/example/myproject/pkg/helpers\""),
        "cmd/main.go: aliased import must have new path:\n{main}"
    );
    assert!(
        !main.contains("pkg/utils\""),
        "cmd/main.go: old pkg/utils must be gone:\n{main}"
    );

    // Control files must be byte-identical
    let config_before = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/go/project/config/config.go"),
    )
    .unwrap();
    let config_after = common::read_file(project, "config/config.go");
    assert_eq!(
        config_after, config_before,
        "config/config.go must be byte-identical"
    );
}

#[test]
fn go_batch_cross_package_moves_both_packages_and_updates_callers() {
    // Move files from TWO different source packages in one batch call.
    // Each source dir needs its own gopls session (different packages).
    //
    //   pkg/utils/format.go → pkg/helpers/format.go  (renames whole utils package)
    //   pkg/models/user.go  → pkg/entities/user.go   (renames whole models package)
    //
    // gopls collaterally moves ALL files in each package:
    //   pkg/utils/validate.go  → pkg/helpers/validate.go
    //   pkg/models/order.go    → pkg/entities/order.go
    //   pkg/models/item.go     → pkg/entities/item.go
    let temp = common::setup_fixture("go/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--source-path",
        project.join("pkg/models/user.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/entities/user.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // Listed files must be at targets
    assert!(
        project.join("pkg/helpers/format.go").exists(),
        "format.go must be at target"
    );
    assert!(
        project.join("pkg/entities/user.go").exists(),
        "user.go must be at target"
    );

    // Collateral: whole utils package moved
    assert!(
        project.join("pkg/helpers/validate.go").exists(),
        "validate.go must be collaterally moved to pkg/helpers"
    );
    assert!(
        !project.join("pkg/utils/validate.go").exists(),
        "validate.go must be gone from pkg/utils"
    );

    // Collateral: whole models package moved
    assert!(
        project.join("pkg/entities/order.go").exists(),
        "order.go must be collaterally moved to pkg/entities"
    );
    assert!(
        project.join("pkg/entities/item.go").exists(),
        "item.go must be collaterally moved to pkg/entities"
    );
    assert!(
        !project.join("pkg/models/order.go").exists(),
        "order.go must be gone from pkg/models"
    );
    assert!(
        !project.join("pkg/models/item.go").exists(),
        "item.go must be gone from pkg/models"
    );

    // Package declarations updated in moved files
    let fmt = common::read_file(project, "pkg/helpers/format.go");
    assert!(
        fmt.contains("package helpers"),
        "format.go: must declare package helpers:\n{fmt}"
    );

    let user = common::read_file(project, "pkg/entities/user.go");
    assert!(
        user.contains("package entities"),
        "user.go: must declare package entities:\n{user}"
    );

    let order = common::read_file(project, "pkg/entities/order.go");
    assert!(
        order.contains("package entities"),
        "order.go: must declare package entities:\n{order}"
    );

    // Callers of utils must now import pkg/helpers
    let service = common::read_file(project, "internal/service/service.go");
    assert!(
        service.contains("\"github.com/example/myproject/pkg/helpers\""),
        "service.go: must import pkg/helpers:\n{service}"
    );
    assert!(
        !service.contains("pkg/utils\""),
        "service.go: old pkg/utils import must be gone:\n{service}"
    );
}
