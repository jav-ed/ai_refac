use super::*;
use std::os::unix::fs::PermissionsExt;

/// A `bun` that takes a moment, notes every call and then creates the package.
fn fake_bun(folder: &Path) -> String {
    let script = folder.join("fake-bun.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\necho call >> \"$PWD/calls.txt\"\nsleep 1\nmkdir -p node_modules/pkg\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script.to_string_lossy().into_owned()
}

#[tokio::test]
async fn two_installs_at_once_run_bun_once() {
    let temp = tempfile::tempdir().unwrap();
    let scripts = temp.path().join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    let bun = fake_bun(temp.path());

    let (first, second) = tokio::join!(
        ensure_installed(&bun, &scripts, &["pkg"]),
        ensure_installed(&bun, &scripts, &["pkg"]),
    );

    first.unwrap();
    second.unwrap();
    let calls = std::fs::read_to_string(scripts.join("calls.txt")).unwrap();
    assert_eq!(calls.lines().count(), 1, "{calls}");
    assert!(scripts.join("node_modules/pkg").exists());
}

#[tokio::test]
async fn present_packages_start_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let scripts = temp.path().join("scripts");
    std::fs::create_dir_all(scripts.join("node_modules/pkg")).unwrap();

    ensure_installed("/nonexistent/bun", &scripts, &["pkg"])
        .await
        .unwrap();

    assert!(!scripts.join(".install.lock").exists());
}
