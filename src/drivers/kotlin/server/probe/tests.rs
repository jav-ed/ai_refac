use super::*;
use std::os::unix::fs::PermissionsExt;

/// The start check of a build that prints its expiry and exits, as the
/// server's EAP builds do after their license date.
#[tokio::test]
async fn a_build_that_has_expired_fails_with_what_it_printed() {
    let dir = tempfile::tempdir().unwrap();
    let executable = dir.path().join("intellij-server");
    std::fs::write(
        &executable,
        "#!/bin/sh\necho 'This build of kotlin-server has expired.' >&2\nexit 7\n",
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();

    let error = format!("{:#}", check_start(&executable).await.unwrap_err());
    // The server's own report is kept, after the explanation.
    assert!(error.contains("status 7"), "{error}");
    assert!(error.contains("has expired"), "{error}");
    assert!(error.contains("refac doctor kotlin"), "{error}");
    assert!(
        error.contains(crate::servers::KOTLIN_SETUP_GUIDE),
        "{error}"
    );
}
