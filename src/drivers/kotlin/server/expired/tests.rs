use super::*;

const SERVER: &str = "/opt/kotlin-server-1.2.3/bin/intellij-server";

fn start_failure(stderr: &str, status: i32) -> anyhow::Error {
    anyhow!(
        "The Kotlin language server closed its connection during initialize. The process exited with status {status}. Its last lines on stderr:\n{stderr}"
    )
}

#[test]
fn an_expired_build_is_explained_with_the_steps_and_the_guide() {
    let error = explain(
        start_failure("This build of kotlin-server has expired.", 7),
        Path::new(SERVER),
    );
    let text = format!("{error:#}");
    assert!(text.contains("this build has expired"), "{text}");
    assert!(text.contains(SERVER), "{text}");
    assert!(text.contains("refac doctor kotlin"), "{text}");
    assert!(text.contains("REFAC_KOTLIN_SERVER"), "{text}");
    assert!(text.contains(KOTLIN_SETUP_GUIDE), "{text}");
    assert!(text.contains("Replacing the build"), "{text}");
    // The server's own words stay, at the end.
    assert!(
        text.ends_with("This build of kotlin-server has expired."),
        "{text}"
    );
}

#[test]
fn the_licensing_exit_code_alone_is_explained_as_a_license_problem() {
    let error = explain(start_failure("something new", 7), Path::new(SERVER));
    let text = format!("{error:#}");
    assert!(text.contains("its license was refused"), "{text}");
    assert!(!text.contains("this build has expired"), "{text}");
    assert!(text.contains("something new"), "{text}");
}

#[test]
fn any_other_start_failure_is_returned_untouched() {
    let error = explain(
        start_failure("java: command not found", 127),
        Path::new(SERVER),
    );
    let text = format!("{error:#}");
    assert!(!text.contains("early-access"), "{text}");
    assert!(text.contains("java: command not found"), "{text}");
}
