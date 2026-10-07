use super::*;

const K: &str = "src/main/kotlin/com/example";

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for file in [
        "util/Helper.kt",
        "util/Other.kt",
        "app/Main.kt",
        "app/Notes.txt",
    ] {
        let path = dir.path().join(K).join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    for java in ["Legacy.java", "old/Old.java"] {
        let java = dir.path().join("src/main/java/com/example").join(java);
        std::fs::create_dir_all(java.parent().unwrap()).unwrap();
        std::fs::write(java, "").unwrap();
    }
    dir
}

fn pair(from: &str, to: &str) -> (String, String) {
    (format!("{K}/{from}"), format!("{K}/{to}"))
}

fn plan(dir: &tempfile::TempDir, moves: &[(String, String)]) -> Result<MovePlan> {
    build(moves, &dir.path().canonicalize().unwrap())
}

#[test]
fn files_for_one_directory_share_a_request() {
    let dir = project();
    let plan = plan(
        &dir,
        &[
            pair("util/Helper.kt", "common/Helper.kt"),
            pair("util/Other.kt", "common/Other.kt"),
        ],
    )
    .unwrap();
    assert_eq!(plan.groups.len(), 1);
    assert_eq!(plan.groups[0].kind, Kind::Move);
    assert_eq!(plan.groups[0].steps.len(), 2);
}

#[test]
fn different_directories_get_their_own_requests_in_order() {
    let dir = project();
    let plan = plan(
        &dir,
        &[
            pair("util/Helper.kt", "common/Helper.kt"),
            pair("app/Main.kt", "launch/Main.kt"),
        ],
    )
    .unwrap();
    assert_eq!(plan.groups.len(), 2);
    assert!(plan.groups[0].steps[0].to.ends_with("common/Helper.kt"));
    assert!(plan.groups[1].steps[0].to.ends_with("launch/Main.kt"));
}

#[test]
fn a_same_directory_rename_is_its_own_kind() {
    let dir = project();
    let plan = plan(&dir, &[pair("util/Helper.kt", "util/Utilities.kt")]).unwrap();
    assert_eq!(plan.groups[0].kind, Kind::Rename);
}

#[test]
fn a_move_with_a_new_name_is_split_into_move_then_rename() {
    let dir = project();
    let plan = plan(&dir, &[pair("util/Helper.kt", "common/Tools.kt")]).unwrap();
    assert_eq!(plan.groups.len(), 2);
    assert_eq!(plan.groups[0].kind, Kind::Move);
    assert!(plan.groups[0].steps[0].to.ends_with("common/Helper.kt"));
    assert_eq!(plan.groups[1].kind, Kind::Rename);
    assert!(plan.groups[1].steps[0].from.ends_with("common/Helper.kt"));
    assert!(plan.groups[1].steps[0].to.ends_with("common/Tools.kt"));
    assert_eq!(plan.notes.len(), 1);
}

#[test]
fn the_temporary_name_must_be_free() {
    let dir = project();
    let taken = dir.path().join(K).join("common/Helper.kt");
    std::fs::create_dir_all(taken.parent().unwrap()).unwrap();
    std::fs::write(taken, "").unwrap();
    let error = plan(&dir, &[pair("util/Helper.kt", "common/Tools.kt")]).unwrap_err();
    assert!(error.to_string().contains("temporary name"), "{error}");
}

#[test]
fn a_directory_move_is_accepted() {
    let dir = project();
    let plan = plan(&dir, &[pair("util", "common")]).unwrap();
    assert!(plan.groups[0].steps[0].is_dir);
    assert_eq!(plan.groups[0].kind, Kind::Rename);
}

#[test]
fn refuses_what_the_server_cannot_do() {
    let dir = project();
    let java = (
        "src/main/java/com/example/Legacy.java".to_string(),
        "src/main/java/com/example/x/Legacy.java".to_string(),
    );
    assert!(
        plan(&dir, &[java])
            .unwrap_err()
            .to_string()
            .contains("Java file")
    );
    let java_dir = (
        "src/main/java/com/example/old".to_string(),
        "src/main/java/com/example/older".to_string(),
    );
    assert!(
        plan(&dir, &[java_dir])
            .unwrap_err()
            .to_string()
            .contains("contains Java sources")
    );
    let text = plan(&dir, &[pair("app/Notes.txt", "util/Notes.txt")]).unwrap_err();
    assert!(text.to_string().contains("not a Kotlin file"), "{text}");
    let renamed = plan(&dir, &[pair("util/Helper.kt", "util/Helper.txt")]).unwrap_err();
    assert!(renamed.to_string().contains(".kt extension"), "{renamed}");
    let outside = (
        "src/main/kotlin/com/example/util/Helper.kt".to_string(),
        "docs/Helper.kt".to_string(),
    );
    assert!(
        plan(&dir, &[outside])
            .unwrap_err()
            .to_string()
            .contains("source root")
    );
    let missing = plan(&dir, &[pair("util/Nope.kt", "common/Nope.kt")]).unwrap_err();
    assert!(missing.to_string().contains("does not exist"), "{missing}");
    let existing = plan(&dir, &[pair("util/Helper.kt", "app/Main.kt")]).unwrap_err();
    assert!(
        existing.to_string().contains("already exists"),
        "{existing}"
    );
}

#[test]
fn refuses_a_move_into_another_module_or_source_set() {
    let dir = project();
    let other_set = (
        "src/main/kotlin/com/example/util/Helper.kt".to_string(),
        "src/test/kotlin/com/example/Helper.kt".to_string(),
    );
    let error = plan(&dir, &[other_set]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("different modules or source sets"),
        "{error}"
    );
    let other_module = (
        "src/main/kotlin/com/example/util/Helper.kt".to_string(),
        "lib/src/main/kotlin/com/example/Helper.kt".to_string(),
    );
    assert!(plan(&dir, &[other_module]).is_err());
    // The kotlin and java folders of one source set belong together.
    let java_folder = (
        "src/main/kotlin/com/example/util/Helper.kt".to_string(),
        "src/main/java/com/example/util/Helper.kt".to_string(),
    );
    assert!(plan(&dir, &[java_folder]).is_ok());
}

#[test]
fn refuses_requests_that_depend_on_each_other() {
    let dir = project();
    let nested = plan(
        &dir,
        &[
            pair("util", "common"),
            pair("util/Helper.kt", "other/Helper.kt"),
        ],
    );
    assert!(nested.unwrap_err().to_string().contains("overlap"));
    let same_target = plan(
        &dir,
        &[
            pair("util/Helper.kt", "x/A.kt"),
            pair("util/Other.kt", "x/A.kt"),
        ],
    );
    assert!(same_target.unwrap_err().to_string().contains("overlap"));
    let into_itself = plan(&dir, &[pair("util", "util/inner")]);
    assert!(into_itself.unwrap_err().to_string().contains("into itself"));
}

#[test]
fn an_empty_request_is_an_error() {
    let dir = project();
    assert!(plan(&dir, &[]).is_err());
}
