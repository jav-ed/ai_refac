//! Which modules a build file names.

use super::dependency::names;
use std::path::Path;

fn names_in(build: &str, other: &str) -> bool {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("app")).unwrap();
    std::fs::write(dir.path().join("app/build.gradle.kts"), build).unwrap();
    names(dir.path(), &dir.path().join("app"), &dir.path().join(other))
}

#[test]
fn a_project_path_and_a_type_safe_accessor_name_a_module() {
    assert!(names_in("implementation(project(\":lib\"))", "lib"));
    assert!(names_in(
        "implementation(project(path = ':libs:core'))",
        "libs/core"
    ));
    assert!(names_in("api(projects.lib)", "lib"));
    assert!(names_in("api(projects.libs.core)", "libs/core"));
    assert!(names_in("api(projects.featureX)", "feature-x"));
}

#[test]
fn a_longer_name_a_comment_free_miss_and_a_missing_build_file_name_nothing() {
    assert!(!names_in("api(projects.library)", "lib"));
    assert!(!names_in("api(project(\":library\"))", "lib"));
    assert!(!names_in("api(someprojects.lib)", "lib"));
    assert!(!names_in("", "lib"));
    let dir = tempfile::tempdir().unwrap();
    assert!(!names(
        dir.path(),
        &dir.path().join("app"),
        Path::new("/elsewhere/lib")
    ));
}
