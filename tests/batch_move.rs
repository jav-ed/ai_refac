mod common;

// Batch move tests — each language gets one or more scenarios that verify
// not just file placement but that cross-imports between the two moved files
// are correctly updated by the driver.
//
// Go same-package batch is the known high-risk case (gopls moves the whole
// package on the first call, so a naïve second call would fail).

// ── TypeScript: unrelated pair ─────────────────────────────────────────────────

#[test]
fn typescript_batch_moves_two_unrelated_files() {
    // Two files with no import relationship move together — placement only.
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("src/utils/string_helpers.ts").to_str().unwrap(),
        "--source-path", project.join("src/utils/math_helpers.ts").to_str().unwrap(),
        "--target-path", project.join("src/moved/string_helpers.ts").to_str().unwrap(),
        "--target-path", project.join("src/moved/math_helpers.ts").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);
    assert!(project.join("src/moved/string_helpers.ts").exists(), "string_helpers.ts must be at target");
    assert!(project.join("src/moved/math_helpers.ts").exists(), "math_helpers.ts must be at target");
    assert!(!project.join("src/utils/string_helpers.ts").exists(), "string_helpers.ts must be gone from source");
    assert!(!project.join("src/utils/math_helpers.ts").exists(), "math_helpers.ts must be gone from source");
}

// ── TypeScript: cross-importing pair ──────────────────────────────────────────

#[test]
fn typescript_batch_updates_cross_import_when_both_files_move_to_same_dir() {
    // task_service.ts imports date_helpers.ts via '../utils/date_helpers'.
    // Both move to src/core/.  Since they land in the same directory, ts-morph
    // must rewrite the import to './date_helpers' (same-dir relative path).
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("src/services/task_service.ts").to_str().unwrap(),
        "--source-path", project.join("src/utils/date_helpers.ts").to_str().unwrap(),
        "--target-path", project.join("src/core/task_service.ts").to_str().unwrap(),
        "--target-path", project.join("src/core/date_helpers.ts").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);
    assert!(project.join("src/core/task_service.ts").exists(), "task_service.ts must be at target");
    assert!(project.join("src/core/date_helpers.ts").exists(), "date_helpers.ts must be at target");
    assert!(!project.join("src/services/task_service.ts").exists(), "task_service.ts must be gone from source");
    assert!(!project.join("src/utils/date_helpers.ts").exists(), "date_helpers.ts must be gone from source");

    let svc = common::read_file(project, "src/core/task_service.ts");

    // Old path must be gone
    assert!(!svc.contains("../utils/date_helpers"), "old import path must be gone:\n{svc}");

    // Both imports (named + dynamic) must resolve to the same-dir sibling
    assert!(
        svc.contains("'./date_helpers'") || svc.contains("\"./date_helpers\""),
        "import must be updated to './date_helpers' (same-dir after batch move):\n{svc}"
    );
}

// ── Python ─────────────────────────────────────────────────────────────────────

#[test]
fn python_batch_moves_two_files_and_updates_all_imports() {
    // validators.py imports formatters.py via `from .formatters import ...`.
    // Both move from myapp/utils/ to myapp/core/.
    //
    // Expected sequence inside Rope (single project, sequential do()):
    //   1. Move formatters.py → Rope rewrites validators.py's relative sibling
    //      import to the absolute path myapp.core.formatters (cross-package move).
    //   2. Move validators.py → Rope moves the file; its import of formatters
    //      is already absolute and still correct, so it stays.
    //
    // After the batch:
    //   myapp/core/validators.py must NOT contain 'myapp.utils.formatters' or '.formatters'
    //   myapp/core/validators.py must contain 'myapp.core.formatters'
    //   External callers (main.py) must point to myapp.core.formatters.
    let temp = common::setup_fixture("python/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("myapp/utils/formatters.py").to_str().unwrap(),
        "--source-path", project.join("myapp/utils/validators.py").to_str().unwrap(),
        "--target-path", project.join("myapp/core/formatters.py").to_str().unwrap(),
        "--target-path", project.join("myapp/core/validators.py").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(project.join("myapp/core/formatters.py").exists(), "formatters.py must be at target");
    assert!(project.join("myapp/core/validators.py").exists(), "validators.py must be at target");
    assert!(!project.join("myapp/utils/formatters.py").exists(), "formatters.py must be gone from source");
    assert!(!project.join("myapp/utils/validators.py").exists(), "validators.py must be gone from source");

    // External caller must point to the new location
    let main = common::read_file(project, "myapp/main.py");
    assert!(!main.contains("myapp.utils.formatters"), "main.py: old path must be gone:\n{main}");
    assert!(main.contains("myapp.core.formatters"), "main.py: new path must be present:\n{main}");

    // validators.py's own import of formatters must also be correct
    let validators = common::read_file(project, "myapp/core/validators.py");
    assert!(
        !validators.contains("myapp.utils.formatters"),
        "validators.py: old formatters path must be gone:\n{validators}"
    );
    assert!(
        !validators.contains("from .formatters") || validators.contains("myapp.core.formatters"),
        "validators.py: import of formatters must resolve to myapp.core.formatters:\n{validators}"
    );
    assert!(
        validators.contains("myapp.core.formatters") || validators.contains("from .formatters"),
        "validators.py: formatters import must still be present in some valid form:\n{validators}"
    );
}

// ── Rust ───────────────────────────────────────────────────────────────────────

#[test]
fn rust_batch_moves_two_files_and_project_still_compiles() {
    // Cross-dir move two files in one batch call.
    // The shim strategy must produce a valid, compilable project for BOTH moves:
    //   src/types.rs  -> src/shared/types.rs
    //   src/error.rs  -> src/shared/error.rs
    //
    // After the move lib.rs must have #[path] for each, shared/mod.rs must
    // re-export both via `pub use crate::...`, and `cargo check` must pass.
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("src/types.rs").to_str().unwrap(),
        "--source-path", project.join("src/error.rs").to_str().unwrap(),
        "--target-path", project.join("src/shared/types.rs").to_str().unwrap(),
        "--target-path", project.join("src/shared/error.rs").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(project.join("src/shared/types.rs").exists(), "types.rs must be at target");
    assert!(project.join("src/shared/error.rs").exists(), "error.rs must be at target");
    assert!(!project.join("src/types.rs").exists(), "types.rs must be gone from source");
    assert!(!project.join("src/error.rs").exists(), "error.rs must be gone from source");

    let lib = common::read_file(project, "src/lib.rs");
    assert!(lib.contains("shared/types.rs"), "lib.rs must have #[path] for types.rs:\n{lib}");
    assert!(lib.contains("shared/error.rs"), "lib.rs must have #[path] for error.rs:\n{lib}");
    assert!(lib.contains("pub mod shared"), "lib.rs must declare pub mod shared:\n{lib}");

    let shared_mod = common::read_file(project, "src/shared/mod.rs");
    assert!(shared_mod.contains("crate::types"), "shared/mod.rs must re-export types:\n{shared_mod}");
    assert!(shared_mod.contains("crate::error"), "shared/mod.rs must re-export error:\n{shared_mod}");

    // The whole project must still compile — two shims must not conflict.
    let check = std::process::Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(project)
        .output()
        .expect("failed to run cargo check");
    assert!(
        check.status.success(),
        "cargo check must pass after batch move:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

// ── Go same-package batch ───────────────────────────────────────────────────────

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
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("pkg/utils/format.go").to_str().unwrap(),
        "--source-path", project.join("pkg/utils/validate.go").to_str().unwrap(),
        "--target-path", project.join("pkg/helpers/format.go").to_str().unwrap(),
        "--target-path", project.join("pkg/helpers/validate.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // File placement
    assert!(project.join("pkg/helpers/format.go").exists(), "format.go must be at target");
    assert!(project.join("pkg/helpers/validate.go").exists(), "validate.go must be at target");
    assert!(!project.join("pkg/utils/format.go").exists(), "format.go must be gone from source");
    assert!(!project.join("pkg/utils/validate.go").exists(), "validate.go must be gone from source");

    // Package declarations updated
    let fmt = common::read_file(project, "pkg/helpers/format.go");
    assert!(fmt.contains("package helpers"), "format.go: package must be helpers:\n{fmt}");
    assert!(!fmt.contains("package utils"), "format.go: old package must be gone:\n{fmt}");

    let val = common::read_file(project, "pkg/helpers/validate.go");
    assert!(val.contains("package helpers"), "validate.go: package must be helpers:\n{val}");

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
    assert!(main.contains("u \"github.com/example/myproject/pkg/helpers\""),
        "cmd/main.go: aliased import must have new path:\n{main}");
    assert!(!main.contains("pkg/utils\""), "cmd/main.go: old pkg/utils must be gone:\n{main}");

    // Control files must be byte-identical
    let config_before = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/go/project/config/config.go")
    ).unwrap();
    let config_after = common::read_file(project, "config/config.go");
    assert_eq!(config_after, config_before, "config/config.go must be byte-identical");
}

// ── Go cross-package batch ─────────────────────────────────────────────────────

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
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("pkg/utils/format.go").to_str().unwrap(),
        "--source-path", project.join("pkg/models/user.go").to_str().unwrap(),
        "--target-path", project.join("pkg/helpers/format.go").to_str().unwrap(),
        "--target-path", project.join("pkg/entities/user.go").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // Listed files must be at targets
    assert!(project.join("pkg/helpers/format.go").exists(), "format.go must be at target");
    assert!(project.join("pkg/entities/user.go").exists(), "user.go must be at target");

    // Collateral: whole utils package moved
    assert!(project.join("pkg/helpers/validate.go").exists(),
        "validate.go must be collaterally moved to pkg/helpers");
    assert!(!project.join("pkg/utils/validate.go").exists(),
        "validate.go must be gone from pkg/utils");

    // Collateral: whole models package moved
    assert!(project.join("pkg/entities/order.go").exists(),
        "order.go must be collaterally moved to pkg/entities");
    assert!(project.join("pkg/entities/item.go").exists(),
        "item.go must be collaterally moved to pkg/entities");
    assert!(!project.join("pkg/models/order.go").exists(),
        "order.go must be gone from pkg/models");
    assert!(!project.join("pkg/models/item.go").exists(),
        "item.go must be gone from pkg/models");

    // Package declarations updated in moved files
    let fmt = common::read_file(project, "pkg/helpers/format.go");
    assert!(fmt.contains("package helpers"), "format.go: must declare package helpers:\n{fmt}");

    let user = common::read_file(project, "pkg/entities/user.go");
    assert!(user.contains("package entities"), "user.go: must declare package entities:\n{user}");

    let order = common::read_file(project, "pkg/entities/order.go");
    assert!(order.contains("package entities"), "order.go: must declare package entities:\n{order}");

    // Callers of utils must now import pkg/helpers
    let service = common::read_file(project, "internal/service/service.go");
    assert!(service.contains("\"github.com/example/myproject/pkg/helpers\""),
        "service.go: must import pkg/helpers:\n{service}");
    assert!(!service.contains("pkg/utils\""),
        "service.go: old pkg/utils import must be gone:\n{service}");
}

// ── Dart ───────────────────────────────────────────────────────────────────────

use std::sync::Mutex;
static DART_LOCK: Mutex<()> = Mutex::new(());

#[test]
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
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--source-path", project.join("lib/src/validator.dart").to_str().unwrap(),
        "--target-path", project.join("lib/src/core/formatter.dart").to_str().unwrap(),
        "--target-path", project.join("lib/src/core/validator.dart").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(project.join("lib/src/core/formatter.dart").exists(), "formatter.dart must be at target");
    assert!(project.join("lib/src/core/validator.dart").exists(), "validator.dart must be at target");
    assert!(!project.join("lib/src/formatter.dart").exists(), "formatter.dart must be gone from source");
    assert!(!project.join("lib/src/validator.dart").exists(), "validator.dart must be gone from source");

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

// ── Markdown ───────────────────────────────────────────────────────────────────

#[test]
fn markdown_batch_moves_two_files_and_updates_all_links() {
    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    // index.md links to both a.md and b.md.
    // a.md also links to b.md (cross-link between the two moved files).
    // Batch move a.md → docs/a.md and b.md → docs/b.md.
    fs::create_dir_all(project.join("docs")).unwrap();
    fs::write(project.join("index.md"),
        "# Index\n\nSee [A](./a.md) and [B](./b.md).\n").unwrap();
    fs::write(project.join("a.md"),
        "# A\n\nSee also [B](./b.md).\n").unwrap();
    fs::write(project.join("b.md"), "# B\n").unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("a.md").to_str().unwrap(),
        "--source-path", project.join("b.md").to_str().unwrap(),
        "--target-path", project.join("docs/a.md").to_str().unwrap(),
        "--target-path", project.join("docs/b.md").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(project.join("docs/a.md").exists(), "a.md must be at docs/a.md");
    assert!(project.join("docs/b.md").exists(), "b.md must be at docs/b.md");
    assert!(!project.join("a.md").exists(), "a.md must be gone from root");
    assert!(!project.join("b.md").exists(), "b.md must be gone from root");

    // External file: both links updated
    let index = fs::read_to_string(project.join("index.md")).unwrap();
    assert!(!index.contains("(./a.md)"), "index.md: old link to a.md must be gone:\n{index}");
    assert!(!index.contains("(./b.md)"), "index.md: old link to b.md must be gone:\n{index}");
    assert!(index.contains("docs/a.md"), "index.md: new link to docs/a.md must be present:\n{index}");
    assert!(index.contains("docs/b.md"), "index.md: new link to docs/b.md must be present:\n{index}");

    // Cross-link inside a.md (a.md → b.md): both landed in docs/, so link becomes ./b.md
    let a = fs::read_to_string(project.join("docs/a.md")).unwrap();
    assert!(
        !a.contains("../b.md") && !a.contains("./b.md") || a.contains("b.md"),
        "a.md must still reference b.md in some valid relative form:\n{a}"
    );
}

// ── Partial failure ────────────────────────────────────────────────────────────

fn gopls_is_available() -> bool {
    // Find gopls the same way the Go driver does.
    let in_path = std::process::Command::new("gopls")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if in_path {
        return true;
    }
    if let Ok(home) = std::env::var("HOME") {
        let gopath_bin = std::path::PathBuf::from(home).join("go/bin/gopls");
        if gopath_bin.exists() {
            return std::process::Command::new(&gopath_bin)
                .arg("version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
    }
    false
}

#[test]
fn partial_failure_reports_success_and_failure_in_same_response() {
    // Skip when gopls is not installed: without it, check_availability() bails
    // before move_files is called, turning a partial failure into a total failure.
    if !gopls_is_available() {
        eprintln!("gopls not found — skipping partial failure test");
        return;
    }

    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    // Markdown move: always succeeds (pure filesystem, no LSP).
    fs::write(project.join("readme.md"), "# Readme\n").unwrap();
    fs::create_dir_all(project.join("docs")).unwrap();

    // Go move: cross-directory, no go.mod → build_go_target_package_path returns
    // Err → move_files returns Err → goes to failed_batches (partial failure).
    fs::create_dir_all(project.join("pkg/utils")).unwrap();
    fs::write(
        project.join("pkg/utils/helpers.go"),
        "package utils\n\nfunc Helper() {}\n",
    )
    .unwrap();
    fs::create_dir_all(project.join("pkg/helpers")).unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("readme.md").to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/helpers.go").to_str().unwrap(),
        "--target-path",
        project.join("docs/readme.md").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/helpers.go").to_str().unwrap(),
    ]);

    // Partial failure: the CLI must still exit 0 (at least one language succeeded).
    assert!(
        output.status.success(),
        "partial failure must exit 0:\nstdout: {}\nstderr: {}",
        common::stdout_text(&output),
        common::stderr_text(&output),
    );

    let stdout = common::stdout_text(&output);

    // Success section must be present for Markdown.
    assert!(
        stdout.contains("Markdown"),
        "stdout must contain Markdown success section:\n{stdout}"
    );

    // Failure section must be present for Go.
    assert!(
        stdout.contains("Failed"),
        "stdout must contain a Failed section:\n{stdout}"
    );

    // Markdown file must have moved.
    assert!(
        project.join("docs/readme.md").exists(),
        "readme.md must be at target after partial success"
    );
    assert!(
        !project.join("readme.md").exists(),
        "readme.md must be gone from source after partial success"
    );
}

#[test]
fn all_failed_batch_exits_nonzero_with_error_message() {
    // When every language batch fails, the CLI must exit non-zero and include
    // a human-readable error.  Trigger this by attempting a Go move without
    // go.mod in a project that has ONLY Go files (no fallback successes).
    //
    // Skip when gopls is not installed (same reason as above).
    if !gopls_is_available() {
        eprintln!("gopls not found — skipping all-failed test");
        return;
    }

    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    fs::create_dir_all(project.join("pkg/utils")).unwrap();
    fs::write(
        project.join("pkg/utils/helpers.go"),
        "package utils\n\nfunc Helper() {}\n",
    )
    .unwrap();
    fs::create_dir_all(project.join("pkg/helpers")).unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/helpers.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/helpers.go").to_str().unwrap(),
    ]);

    assert!(
        !output.status.success(),
        "all-failed batch must exit non-zero"
    );

    let stderr = common::stderr_text(&output);
    assert!(
        !stderr.is_empty() || !common::stdout_text(&output).is_empty(),
        "some error output must be present when all batches fail"
    );
}

// ── Mixed-language batch ───────────────────────────────────────────────────────

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

    fs::write(project.join("tsconfig.json"),
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#
    ).unwrap();
    fs::write(project.join("src/lib.ts"),
        "export const greeting = \"hello\";\n").unwrap();
    fs::write(project.join("src/consumer.ts"),
        "import { greeting } from './lib';\nconsole.log(greeting);\n").unwrap();
    fs::write(project.join("docs/index.md"),
        "# Docs\n\nSee [notes](./notes.md).\n").unwrap();
    fs::write(project.join("docs/notes.md"), "# Notes\n").unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path", project.to_str().unwrap(),
        "--source-path", project.join("src/lib.ts").to_str().unwrap(),
        "--source-path", project.join("docs/notes.md").to_str().unwrap(),
        "--target-path", project.join("src/utils/lib.ts").to_str().unwrap(),
        "--target-path", project.join("archive/notes.md").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    // TypeScript: file placed, consumer.ts import updated
    assert!(project.join("src/utils/lib.ts").exists(), "lib.ts must be at target");
    assert!(!project.join("src/lib.ts").exists(), "lib.ts must be gone from source");

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
    assert!(project.join("archive/notes.md").exists(), "notes.md must be at target");
    assert!(!project.join("docs/notes.md").exists(), "notes.md must be gone from source");

    let index = fs::read_to_string(project.join("docs/index.md")).unwrap();
    assert!(!index.contains("(./notes.md)"), "index.md: old link must be gone:\n{index}");
    assert!(
        index.contains("../archive/notes.md") || index.contains("archive/notes.md"),
        "index.md: link must be updated to archive/notes.md:\n{index}"
    );
}
