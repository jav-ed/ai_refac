mod common;

// Fixture: tests/fixtures/python/project/  (21 files)
//
// Move under test:
//   myapp/utils/formatters.py  ->  myapp/core/formatters.py
//
// IMPORTANT: myapp/core/__init__.py must pre-exist in the fixture.
// Without it, Rope moves the file but silently skips all import rewrites.
//
// Patterns Rope WILL update (positive assertions):
//   main.py                  absolute multi-name from-import
//   utils/__init__.py        relative re-export -> rewritten to absolute
//   utils/validators.py      relative sibling import -> rewritten to absolute
//   utils/converters.py      absolute direct import
//   models/record.py         absolute import + aliased import (alias preserved)
//   services/user_service.py absolute direct import
//   api/routes.py            absolute direct import (multi-file coverage)
//   api/handlers.py          module-level `import X as Y`
//   tests/test_formatters.py absolute import in test file
//   conftest.py              absolute import outside package
//
// Patterns Rope will NOT update (negative assertions — document the limitation):
//   models/order.py          `from ..utils import ...` (indirect via __init__)
//   services/order_service.py `from myapp.utils import ...` (indirect via __init__)
//
// Control files (byte-identical before/after):
//   config.py, models/base.py

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("myapp/utils/formatters.py").to_str().unwrap(),
        "--target-path",
        project.join("myapp/core/formatters.py").to_str().unwrap(),
    ])
}

// ── file placement ────────────────────────────────────────────────────────────

#[test]
fn python_move_places_file_at_target_and_removes_source() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    assert!(project.join("myapp/core/formatters.py").exists(), "file must exist at target");
    assert!(!project.join("myapp/utils/formatters.py").exists(), "file must be gone from source");
}

// ── positive assertions ───────────────────────────────────────────────────────

#[test]
fn python_move_updates_direct_imports_across_all_files() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let cases = [
        "myapp/main.py",
        "myapp/utils/converters.py",
        "myapp/services/user_service.py",
        "myapp/api/routes.py",
        "tests/test_formatters.py",
        "conftest.py",
    ];
    for rel in &cases {
        let f = common::read_file(project, rel);
        assert!(
            !f.contains("myapp.utils.formatters"),
            "old path must be gone in {rel}:\n{f}"
        );
        assert!(
            f.contains("myapp.core.formatters"),
            "new path must be present in {rel}:\n{f}"
        );
    }
}

#[test]
fn python_move_updates_relative_sibling_import_to_absolute() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // utils/validators.py had `from .formatters import ...`
    // Rope rewrites relative sibling imports to absolute after a cross-package move.
    let f = common::read_file(project, "myapp/utils/validators.py");
    assert!(!f.contains("from .formatters"), "old relative import must be gone:\n{f}");
    assert!(f.contains("myapp.core.formatters"), "new absolute path must be present:\n{f}");
}

#[test]
fn python_move_rewrites_init_reexport_from_relative_to_absolute() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // utils/__init__.py had `from .formatters import format_date, format_currency`
    // After the move Rope rewrites it to an absolute import.
    let f = common::read_file(project, "myapp/utils/__init__.py");
    assert!(!f.contains("from .formatters"), "old relative re-export must be gone:\n{f}");
    assert!(f.contains("myapp.core.formatters"), "new absolute re-export must be present:\n{f}");
}

#[test]
fn python_move_preserves_alias_in_aliased_import() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // models/record.py has `from myapp.utils.formatters import format_date as fd`
    let f = common::read_file(project, "myapp/models/record.py");
    assert!(!f.contains("myapp.utils.formatters"), "old path must be gone:\n{f}");
    assert!(f.contains("myapp.core.formatters"), "new path must be present:\n{f}");
    assert!(f.contains("as fd"), "alias 'fd' must be preserved:\n{f}");
}

#[test]
fn python_move_updates_module_level_aliased_import() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // api/handlers.py has `import myapp.utils.formatters as fmt`
    let f = common::read_file(project, "myapp/api/handlers.py");
    assert!(!f.contains("myapp.utils.formatters"), "old module path must be gone:\n{f}");
    assert!(f.contains("myapp.core.formatters"), "new module path must be present:\n{f}");
    assert!(f.contains("as fmt"), "alias 'fmt' must be preserved:\n{f}");
}

// ── negative assertions (Rope limitation: indirect imports not updated) ────────

#[test]
fn python_move_does_not_update_indirect_imports_via_init_reexport() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    // These files import via __init__ re-export, not directly from formatters.py.
    // Rope does not trace through __init__ and leaves them unchanged.
    // They still work at runtime because utils/__init__.py is updated.
    let order = common::read_file(project, "myapp/models/order.py");
    assert!(
        order.contains("from ..utils import"),
        "indirect relative import must be unchanged in order.py:\n{order}"
    );

    let order_svc = common::read_file(project, "myapp/services/order_service.py");
    assert!(
        order_svc.contains("from myapp.utils import"),
        "indirect absolute import must be unchanged in order_service.py:\n{order_svc}"
    );
}

// ── control files ─────────────────────────────────────────────────────────────

#[test]
fn python_move_does_not_touch_unrelated_files() {
    let temp = common::setup_fixture("python/project");
    let project = temp.path();

    let config_before = common::read_file(project, "myapp/config.py");
    let base_before = common::read_file(project, "myapp/models/base.py");

    common::assert_move_succeeded(&run_move(project));

    assert_eq!(common::read_file(project, "myapp/config.py"), config_before,
        "config.py must be byte-identical after move");
    assert_eq!(common::read_file(project, "myapp/models/base.py"), base_before,
        "models/base.py must be byte-identical after move");
}
