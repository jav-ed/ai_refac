use super::*;

#[test]
fn the_gradle_daemon_of_the_import_is_told_to_stop_itself() {
    assert_eq!(
        java_tool_options(None, 10_000),
        "-Dorg.gradle.daemon.idletimeout=10000"
    );
    assert_eq!(
        java_tool_options(Some("  "), 10_000),
        "-Dorg.gradle.daemon.idletimeout=10000"
    );
}

#[test]
fn options_the_user_already_set_are_kept() {
    assert_eq!(
        java_tool_options(Some("-Xmx2g"), 10_000),
        "-Xmx2g -Dorg.gradle.daemon.idletimeout=10000"
    );
}

#[test]
fn a_caller_who_keeps_the_gradle_daemon_gets_the_time_he_asked_for() {
    assert_eq!(
        java_tool_options(Some("-Xmx2g"), 300_000),
        "-Xmx2g -Dorg.gradle.daemon.idletimeout=300000"
    );
}
