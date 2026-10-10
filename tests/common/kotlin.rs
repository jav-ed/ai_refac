//! Helpers for the real Kotlin server tests. Those tests are `#[ignore]`d;
//! running them without the server is a setup mistake, so they fail loudly.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Stop with the install hint when the Kotlin language server is not set up.
pub fn require_server() {
    assert!(
        std::env::var_os("REFAC_KOTLIN_SERVER").is_some(),
        "REFAC_KOTLIN_SERVER is not set. Install the Kotlin language server first (see Project_Manag/Docs/Setup/kotlin_Server.md) and point REFAC_KOTLIN_SERVER at it."
    );
}

/// Every source file below `root` with its bytes, to prove that a failed
/// refactor left the project exactly as it was. Gradle and the server write
/// their own state into the project, which is not part of the comparison.
pub fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy();
            if path.is_dir() {
                if !matches!(name.as_ref(), ".gradle" | ".kotlin" | ".idea" | "build") {
                    walk(root, &path, out);
                }
            } else {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// The judge of a refactor: the project still compiles, Kotlin and Java.
/// `--no-daemon`: a Gradle daemon would stay in memory (about 0.5 GB) for three
/// hours after the test, and nothing a test starts may outlive it.
pub fn assert_compiles(project: &Path, tasks: &[&str]) {
    compile(project, tasks, &["--no-daemon"]);
}

/// Run Gradle `tasks` in `project` with `extra` arguments and fail with its
/// output when the project does not compile.
pub fn compile(project: &Path, tasks: &[&str], extra: &[&str]) {
    let output = Command::new("./gradlew")
        .args(tasks)
        .args(["--console=plain", "-q"])
        .args(extra)
        .current_dir(project)
        .output()
        .expect("failed to run ./gradlew");
    assert!(
        output.status.success(),
        "the refactored project does not compile:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Android builds need the SDK; a missing one is a setup mistake, not a skip.
pub fn require_android_sdk() {
    assert!(
        std::env::var_os("ANDROID_HOME").is_some(),
        "ANDROID_HOME is not set. The Android tests compile the fixture, which needs the Android SDK (platform 36, build-tools 36.0.0)."
    );
}
