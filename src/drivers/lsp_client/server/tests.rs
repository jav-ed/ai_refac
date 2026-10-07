use super::*;

#[test]
fn every_language_the_drivers_use_has_a_profile() {
    assert_eq!(Server::for_language("dart").unwrap(), Server::Dart);
    assert_eq!(Server::for_language("go").unwrap(), Server::Go);
    assert_eq!(Server::for_language("rust").unwrap(), Server::Rust);
    let error = Server::for_language("cobol").unwrap_err().to_string();
    assert!(error.contains("cobol"), "{error}");
}

#[test]
fn each_server_gets_the_capability_its_readiness_signal_needs() {
    let go = Server::Go.capabilities(false);
    assert_eq!(go["window"]["workDoneProgress"], true);
    let rust = Server::Rust.capabilities(false);
    assert_eq!(rust["experimental"]["serverStatusNotification"], true);
    // The Dart server reports `$/analyzerStatus` only to a client that does not
    // ask for work-done progress.
    let dart = Server::Dart.capabilities(true);
    assert!(dart.get("window").is_none());
    assert_eq!(dart["workspace"]["fileOperations"]["willRename"], true);
    assert!(go["workspace"].get("fileOperations").is_none());
}

#[test]
fn the_readiness_notifications_are_the_ones_the_session_keeps() {
    assert_eq!(Server::Dart.kept_notifications(), ["$/analyzerStatus"]);
    assert_eq!(Server::Go.kept_notifications(), ["$/progress"]);
    assert_eq!(
        Server::Rust.kept_notifications(),
        ["experimental/serverStatus"]
    );
}
