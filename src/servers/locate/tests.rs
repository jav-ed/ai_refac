use super::probe::run_with_timeout;
use super::*;
use crate::drivers::lsp_client::Server as Profile;
use crate::servers::{Launch, Version};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{Duration, Instant};

/// A "server" that is a shell script in `<project>/bin`, found through the
/// folder search because no such command exists on PATH.
static FAKE: Server = Server {
    languages: &["fake"],
    language: "Fake",
    name: "fakeserver",
    used_for: "tests",
    env_var: "REFAC_TEST_FAKE_SERVER_UNSET",
    find: Find::Executable {
        candidates: &[Candidate {
            executable: "refac-fake-server",
            version: Version::Own(&["--version"]),
        }],
        folders: |project| vec![project.join("bin")],
        launch: Launch {
            profile: Profile::Go,
            args: &[],
        },
    },
    requirements: &[],
    install: &["install the fake server"],
};

fn script(dir: &Path, body: &str) {
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    let path = dir.join("bin/refac-fake-server");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn a_server_in_a_known_folder_is_found_and_its_version_read() {
    let project = tempfile::tempdir().unwrap();
    script(project.path(), "echo 'fake 1.2.3'; echo second line");
    let found = locate_with(&FAKE, project.path(), None).unwrap();
    assert_eq!(found.version, "fake 1.2.3");
    assert_eq!(
        found.executable,
        project.path().join("bin/refac-fake-server")
    );
    assert_eq!(found.via, project.path().join("bin").display().to_string());
}

#[test]
fn a_missing_server_lists_every_place_and_names_the_help_command() {
    let project = tempfile::tempdir().unwrap();
    let error = locate_with(&FAKE, project.path(), None).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("was not found"), "{text}");
    assert!(
        text.contains("$REFAC_TEST_FAKE_SERVER_UNSET: not set"),
        "{text}"
    );
    assert!(
        text.contains("PATH: no command named `refac-fake-server`"),
        "{text}"
    );
    assert!(
        text.contains("bin/refac-fake-server: no such file"),
        "{text}"
    );
    assert!(text.contains("no server has to be running"), "{text}");
    assert!(text.contains("Run `refac doctor fake`"), "{text}");
}

#[test]
fn a_server_that_exists_but_fails_is_broken_not_missing() {
    let project = tempfile::tempdir().unwrap();
    script(
        project.path(),
        "echo \"error: Unknown binary 'x'\" >&2; exit 1",
    );
    let error = locate_with(&FAKE, project.path(), None).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("installed but does not work"), "{text}");
    assert!(text.contains("Unknown binary 'x'"), "{text}");
    assert!(!text.contains("was not found"), "{text}");
}

#[test]
fn a_probe_that_hangs_is_killed_at_the_timeout() {
    let began = Instant::now();
    let error =
        run_with_timeout(Command::new("sleep").arg("30"), Duration::from_millis(200)).unwrap_err();
    assert!(error.to_string().contains("did not finish"), "{error}");
    assert!(began.elapsed() < Duration::from_secs(5));
}

#[test]
fn an_environment_variable_beats_every_other_place_and_a_wrong_one_is_final() {
    static WITH_VARIABLE: Server = Server {
        languages: &["fake"],
        language: "Fake",
        name: "fakeserver",
        used_for: "tests",
        env_var: "REFAC_TEST_FAKE_SERVER_SET",
        find: Find::Executable {
            candidates: &[Candidate {
                executable: "refac-fake-server",
                version: Version::Own(&["--version"]),
            }],
            folders: |project| vec![project.join("bin")],
            launch: Launch {
                profile: Profile::Go,
                args: &[],
            },
        },
        requirements: &[],
        install: &[],
    };
    let project = tempfile::tempdir().unwrap();
    // A good server in the folder search must not rescue a wrong variable.
    script(project.path(), "echo 'fake from folder'");
    let wrong = Some(OsString::from("/definitely/not/there"));
    let error = locate_with(&WITH_VARIABLE, project.path(), wrong)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("$REFAC_TEST_FAKE_SERVER_SET is set to /definitely/not/there"),
        "{error}"
    );
    assert!(!error.contains("bin/refac-fake-server"), "{error}");

    let own = project.path().join("own-server");
    std::fs::write(&own, "#!/bin/sh\necho 'fake from variable'\n").unwrap();
    std::fs::set_permissions(&own, std::fs::Permissions::from_mode(0o755)).unwrap();
    let found = locate_with(&WITH_VARIABLE, project.path(), Some(own.clone().into())).unwrap();
    assert_eq!(found.version, "fake from variable");
    assert_eq!(found.via, "$REFAC_TEST_FAKE_SERVER_SET");
}

#[test]
fn a_downloaded_folder_needs_its_build_file_and_executable() {
    static FOLDER: Server = Server {
        languages: &["folder"],
        language: "Folder",
        name: "foldered",
        used_for: "tests",
        env_var: "REFAC_TEST_FOLDER_SERVER",
        find: Find::Folder {
            executable: "bin/run",
            build_file: "build.txt",
        },
        requirements: &[],
        install: &[],
    };
    let project = tempfile::tempdir().unwrap();
    let missing = locate_with(&FOLDER, project.path(), None)
        .unwrap_err()
        .to_string();
    assert!(
        missing.contains("$REFAC_TEST_FOLDER_SERVER: not set"),
        "{missing}"
    );

    let folder = tempfile::tempdir().unwrap();
    let set = || Some(OsString::from(folder.path()));
    let error = locate_with(&FOLDER, project.path(), set())
        .unwrap_err()
        .to_string();
    assert!(error.contains("no build.txt in it"), "{error}");

    std::fs::write(folder.path().join("build.txt"), "ILS-1.2\n").unwrap();
    let error = locate_with(&FOLDER, project.path(), set())
        .unwrap_err()
        .to_string();
    assert!(error.contains("bin/run is missing"), "{error}");

    std::fs::create_dir(folder.path().join("bin")).unwrap();
    std::fs::write(folder.path().join("bin/run"), "").unwrap();
    let found = locate_with(&FOLDER, project.path(), set()).unwrap();
    assert_eq!(found.version, "ILS-1.2");
    assert_eq!(found.folder.as_deref(), Some(folder.path()));
}
