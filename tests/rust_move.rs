mod common;

// Fixture: tests/fixtures/rust/project/  (21 files)
//
// Move under test:
//   src/types.rs  ->  src/shared/types.rs
//
// IMPORTANT — Rust cross-directory move uses a SHIM strategy, not caller rewrites:
//
//   1. src/types.rs is physically moved to src/shared/types.rs
//   2. src/shared/mod.rs is CREATED containing:
//        pub use crate::types;    (alias that keeps `crate::types` working)
//      Note: no `mod types;` here — lib.rs already owns that declaration via #[path].
//   3. src/lib.rs `pub mod types;` is patched to:
//        #[path = "shared/types.rs"]
//        pub mod types;
//      and `pub mod shared;` is appended
//   4. ALL caller files (utils.rs, models/order.rs, services/order.rs, etc.)
//      are left BYTE-IDENTICAL — `crate::types` still resolves via the alias
//
// Tests therefore assert structure and shim content, NOT import rewrites.

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("src/types.rs").to_str().unwrap(),
        "--target-path",
        project.join("src/shared/types.rs").to_str().unwrap(),
    ])
}

// ── file placement ────────────────────────────────────────────────────────────

#[test]
fn rust_move_places_file_at_target_and_removes_source() {
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    assert!(project.join("src/shared/types.rs").exists(), "file must exist at target");
    assert!(!project.join("src/types.rs").exists(), "file must be gone from source");
}

// ── shim: intermediate module ─────────────────────────────────────────────────

#[test]
fn rust_move_creates_intermediate_module_file() {
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    assert!(
        project.join("src/shared/mod.rs").exists(),
        "src/shared/mod.rs must be created by the move"
    );

    let shim = common::read_file(project, "src/shared/mod.rs");
    // The shim re-exports crate::types so callers that use `crate::types::X` still work.
    // There is no separate `mod types;` here — the crate root lib.rs already owns that
    // declaration (with a `#[path]` redirect), so shared/mod.rs only needs the re-export.
    assert!(
        shim.contains("crate::types"),
        "shared/mod.rs must re-export via `pub use crate::types`:\n{shim}"
    );
}

// ── shim: lib.rs patch ────────────────────────────────────────────────────────

#[test]
fn rust_move_patches_lib_rs_with_path_attribute() {
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let lib = common::read_file(project, "src/lib.rs");
    assert!(
        lib.contains("#[path"),
        "lib.rs must contain a #[path] attribute after move:\n{lib}"
    );
    assert!(
        lib.contains("shared/types.rs"),
        "lib.rs #[path] must point to shared/types.rs:\n{lib}"
    );
    assert!(
        lib.contains("pub mod shared"),
        "lib.rs must declare `pub mod shared` after move:\n{lib}"
    );
}

// ── shim: callers unchanged ───────────────────────────────────────────────────

#[test]
fn rust_move_leaves_all_caller_files_unchanged() {
    // The shim strategy means callers are never touched.
    // crate::types still resolves through the alias in shared/mod.rs.
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();

    let snapshots: Vec<(&str, String)> = vec![
        "src/error.rs",
        "src/config.rs",
        "src/prelude.rs",
        "src/utils.rs",
        "src/utils/formatter.rs",
        "src/utils/parser.rs",
        "src/utils/validator.rs",
        "src/models/user.rs",
        "src/models/order.rs",
        "src/services/user.rs",
        "src/services/order.rs",
        "src/core.rs",
        "src/api.rs",
        "src/api/handler.rs",
        "src/api/router.rs",
    ]
    .into_iter()
    .map(|p| (p, common::read_file(project, p)))
    .collect();

    common::assert_move_succeeded(&run_move(project));

    for (rel, before) in &snapshots {
        let after = common::read_file(project, rel);
        assert_eq!(
            after, *before,
            "{rel} must be byte-identical after move (shim strategy leaves callers untouched)"
        );
    }
}
