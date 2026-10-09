use super::*;

#[test]
fn the_gradle_daemon_of_the_import_is_told_to_stop_itself() {
    assert_eq!(
        java_tool_options(None),
        "-Dorg.gradle.daemon.idletimeout=10000"
    );
    assert_eq!(
        java_tool_options(Some("  ")),
        "-Dorg.gradle.daemon.idletimeout=10000"
    );
}

#[test]
fn options_the_user_already_set_are_kept() {
    assert_eq!(
        java_tool_options(Some("-Xmx2g")),
        "-Xmx2g -Dorg.gradle.daemon.idletimeout=10000"
    );
}
