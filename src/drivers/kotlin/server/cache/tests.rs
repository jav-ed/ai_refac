use super::*;

fn warm_in(dir: &Path) -> Option<Warm> {
    Some(Warm::at(dir.join("kotlin-test")))
}

fn text(dir: &SystemDir, name: &str) -> Option<String> {
    fs::read_to_string(dir.path().join(name)).ok()
}

#[test]
fn the_cache_is_off_when_the_setting_says_so() {
    assert!(
        locate("1", Some("off"), None, Some("/home/me".into()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn an_empty_setting_is_an_error() {
    let error = locate("1", Some(""), None, None).err().unwrap().to_string();
    assert!(error.contains(CACHE_ENV), "{error}");
}

#[test]
fn the_place_is_the_setting_then_xdg_then_home() {
    let place = |configured, xdg: Option<&str>, home: Option<&str>| {
        locate("263", configured, xdg.map(Into::into), home.map(Into::into))
            .unwrap()
            .map(|warm| warm.root)
    };
    assert_eq!(
        place(Some("/c"), Some("/x"), Some("/h")),
        Some(PathBuf::from("/c/kotlin-263"))
    );
    assert_eq!(
        place(None, Some("/x"), Some("/h")),
        Some(PathBuf::from("/x/refac/kotlin-263"))
    );
    assert_eq!(
        place(None, None, Some("/h")),
        Some(PathBuf::from("/h/.cache/refac/kotlin-263"))
    );
    assert_eq!(
        place(None, Some(""), Some("/h")),
        Some(PathBuf::from("/h/.cache/refac/kotlin-263"))
    );
    assert_eq!(place(None, None, None), None);
}

#[test]
fn a_run_starts_with_what_the_run_before_it_kept() {
    let cache = tempfile::tempdir().unwrap();
    let first = SystemDir::with(warm_in(cache.path())).unwrap();
    assert_eq!(text(&first, "index"), None);
    fs::create_dir(first.path().join("rocks")).unwrap();
    fs::write(first.path().join("rocks/index"), "libraries").unwrap();
    first.keep().unwrap();

    let second = SystemDir::with(warm_in(cache.path())).unwrap();
    assert_eq!(text(&second, "rocks/index").as_deref(), Some("libraries"));
}

#[test]
fn a_run_that_is_not_kept_leaves_the_warm_directory_as_it_was() {
    let cache = tempfile::tempdir().unwrap();
    let first = SystemDir::with(warm_in(cache.path())).unwrap();
    fs::write(first.path().join("index"), "one").unwrap();
    first.keep().unwrap();

    let second = SystemDir::with(warm_in(cache.path())).unwrap();
    fs::write(second.path().join("index"), "two").unwrap();
    drop(second);

    let third = SystemDir::with(warm_in(cache.path())).unwrap();
    assert_eq!(text(&third, "index").as_deref(), Some("one"));
}

#[test]
fn the_run_that_is_kept_replaces_the_warm_directory_and_leaves_nothing_else() {
    let cache = tempfile::tempdir().unwrap();
    for version in ["one", "two"] {
        let run = SystemDir::with(warm_in(cache.path())).unwrap();
        fs::write(run.path().join("index"), version).unwrap();
        fs::write(run.path().join(format!("only-in-{version}")), "").unwrap();
        run.keep().unwrap();
    }
    let last = SystemDir::with(warm_in(cache.path())).unwrap();
    assert_eq!(text(&last, "index").as_deref(), Some("two"));
    // What the first run left stays: a run adds to what is known.
    assert!(last.path().join("only-in-one").exists());

    let mut names: Vec<String> = fs::read_dir(cache.path().join("kotlin-test"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, [".lock", "warm"]);
}

#[test]
fn a_cache_over_the_limit_is_deleted_and_said_so() {
    let cache = tempfile::tempdir().unwrap();
    let small = SystemDir::with(warm_in(cache.path())).unwrap();
    fs::write(small.path().join("index"), "x").unwrap();
    small.keep().unwrap();

    let big = SystemDir::with(warm_in(cache.path())).unwrap();
    fs::write(big.path().join("big"), vec![0u8; 4096]).unwrap();
    let error = big.keep_with_limit(1024).err().unwrap().to_string();
    assert!(error.contains("over the limit"), "{error}");

    let next = SystemDir::with(warm_in(cache.path())).unwrap();
    assert_eq!(text(&next, "index"), None);
}

#[test]
fn a_link_in_the_warm_directory_is_refused() {
    let cache = tempfile::tempdir().unwrap();
    let warm = cache.path().join("kotlin-test/warm");
    fs::create_dir_all(&warm).unwrap();
    std::os::unix::fs::symlink("/etc/hostname", warm.join("link")).unwrap();

    let error = SystemDir::with(warm_in(cache.path())).err().unwrap();
    let message = format!("{error:#}");
    assert!(message.contains("neither a file nor a folder"), "{message}");
    assert!(message.contains(CACHE_ENV), "{message}");
}

#[test]
fn without_a_cache_every_run_starts_empty() {
    let run = SystemDir::with(None).unwrap();
    assert!(fs::read_dir(run.path()).unwrap().next().is_none());
    run.keep().unwrap();
}
