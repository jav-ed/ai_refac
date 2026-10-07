use super::*;

fn install_dir(build_txt: Option<&str>, with_executable: bool) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    if let Some(text) = build_txt {
        std::fs::write(dir.path().join("build.txt"), text).unwrap();
    }
    if with_executable {
        std::fs::create_dir(dir.path().join("bin")).unwrap();
        std::fs::write(dir.path().join("bin/intellij-server"), "").unwrap();
    }
    dir
}

#[test]
fn a_missing_setting_explains_the_install() {
    let error = locate_in(None).err().unwrap().to_string();
    assert!(error.contains(SERVER_ENV), "{error}");
    assert!(error.contains("not set"), "{error}");
    assert!(error.contains("refac doctor kotlin"), "{error}");
}

#[test]
fn a_folder_without_build_txt_is_refused() {
    let dir = install_dir(None, true);
    let error = locate_in(Some(dir.path().into()))
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("no build.txt"), "{error}");
}

#[test]
fn a_foreign_build_is_refused() {
    let dir = install_dir(Some("IC-263.1"), true);
    let error = locate_in(Some(dir.path().into()))
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("expected ILS-"), "{error}");
}

#[test]
fn a_missing_executable_is_refused() {
    let dir = install_dir(Some("ILS-263.6379.0\n"), false);
    let error = locate_in(Some(dir.path().into()))
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("intellij-server"), "{error}");
}

#[test]
fn a_complete_install_reports_its_build() {
    let dir = install_dir(Some("ILS-263.6379.0\n"), true);
    let install = locate_in(Some(dir.path().into())).unwrap();
    assert_eq!(install.build, "263.6379.0");
}

#[test]
fn the_timeout_defaults_and_rejects_nonsense() {
    assert_eq!(parse_timeout(None).unwrap(), Duration::from_secs(600));
    assert_eq!(parse_timeout(Some("90")).unwrap(), Duration::from_secs(90));
    assert!(parse_timeout(Some("0")).is_err());
    assert!(parse_timeout(Some("soon")).is_err());
}
