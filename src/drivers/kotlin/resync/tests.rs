use super::*;

fn path(name: &str) -> PathBuf {
    PathBuf::from("/project").join(name)
}

fn types(changes: &[Value]) -> Vec<(String, u64)> {
    changes
        .iter()
        .map(|change| {
            (
                change["uri"]
                    .as_str()
                    .unwrap()
                    .replace("file:///project/", ""),
                change["type"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn an_edited_file_is_a_change_at_its_own_path() {
    let changes = changes(&[path("A.kt"), path("B.java")], &[]).unwrap();

    assert_eq!(
        types(&changes),
        vec![("A.kt".to_string(), 2), ("B.java".to_string(), 2)]
    );
}

#[test]
fn a_moved_file_is_deleted_created_and_changed_at_its_new_path() {
    let moves = [(path("Old.kt"), path("New.kt"))];

    let changes = changes(&[path("Old.kt"), path("User.kt")], &moves).unwrap();

    assert_eq!(
        types(&changes),
        vec![
            ("Old.kt".to_string(), 3),
            ("New.kt".to_string(), 1),
            ("New.kt".to_string(), 2),
            ("User.kt".to_string(), 2),
        ]
    );
}

#[test]
fn only_kotlin_and_java_files_are_shown_to_the_server_again() {
    assert!(is_source(&path("A.kt")));
    assert!(is_source(&path("B.java")));
    assert!(!is_source(&path("layout.xml")));
}
