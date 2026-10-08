use super::*;

#[test]
fn a_helper_script_of_the_checkout_is_found_whatever_the_current_directory() {
    // The test binary is built into `target/` (found through the folders above
    // the executable) or into another CARGO_TARGET_DIR (found through the
    // checkout the binary was built from); either way the folder the command
    // runs in does not matter.
    let path = resolve_resource_path("scripts/ts_refactor.ts").unwrap();
    assert!(path.is_file(), "{}", path.display());
    assert!(path.ends_with("scripts/ts_refactor.ts"));
}

#[test]
fn a_missing_resource_names_every_place_that_was_looked_at() {
    let error = resolve_resource_path("scripts/no_such_helper.ts")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Could not find scripts/no_such_helper.ts"),
        "{error}"
    );
    assert!(error.contains("folders above it"), "{error}");
    assert!(
        error.contains("checkout this binary was built from"),
        "{error}"
    );
    assert!(error.contains("current directory"), "{error}");
}
