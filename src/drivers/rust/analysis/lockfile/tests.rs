use super::*;

#[test]
fn a_lock_file_written_after_watching_is_removed_and_an_old_one_stays() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("Cargo.toml"), "").unwrap();
    std::fs::write(temp.path().join("Cargo.toml"), "").unwrap();
    std::fs::write(temp.path().join("Cargo.lock"), "old").unwrap();

    {
        let _watch = NewLockfiles::watch(&project);
        std::fs::write(project.join("Cargo.lock"), "new").unwrap();
        std::fs::write(temp.path().join("Cargo.lock"), "changed").unwrap();
    }

    assert!(!project.join("Cargo.lock").exists());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("Cargo.lock")).unwrap(),
        "changed"
    );
}

#[test]
fn a_folder_without_a_manifest_is_not_watched() {
    let temp = tempfile::tempdir().unwrap();
    let watch = NewLockfiles::watch(temp.path());
    assert!(
        watch
            .missing
            .iter()
            .all(|lock| lock.parent().unwrap().join("Cargo.toml").exists())
    );
}
