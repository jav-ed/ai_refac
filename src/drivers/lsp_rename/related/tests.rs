use super::*;
use crate::drivers::lsp_rename::discover::discover;
use crate::drivers::lsp_rename::test_language::{Plain, WordServer};
use crate::drivers::symbol_scan;

const SOURCE: &str = "def area\ncall area\nother  thing\n";

fn project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.pl");
    std::fs::write(&file, SOURCE).unwrap();
    (dir, file)
}

async fn candidate(server: &mut WordServer, file: &Path) -> Candidate {
    let occurrences = symbol_scan::occurrences(
        SOURCE,
        "area",
        None,
        None,
        |c| c == '_' || c.is_alphanumeric(),
        symbol_scan::line_column(SOURCE),
    )
    .unwrap();
    server.docs.insert(file.to_path_buf(), SOURCE.to_string());
    discover(
        server,
        &Plain,
        file,
        SOURCE,
        &occurrences,
        "area",
        "surface",
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn a_rename_that_stays_inside_its_references_is_one_group() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    let candidate = candidate(&mut server, &file).await;

    let groups = groups(&mut server, &Plain, &candidate, &file, "area")
        .await
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].references.len(), 2);
}

#[tokio::test]
async fn an_edit_at_another_symbol_joins_the_rename_as_a_group_of_its_own() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    // Like an interface method rename that also renames the implementer.
    server.extra_edits = vec![(file.clone(), 2, 0, 5)];
    let candidate = candidate(&mut server, &file).await;

    let groups = groups(&mut server, &Plain, &candidate, &file, "area")
        .await
        .unwrap();

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[1].references.len(), 1);
    assert_eq!(groups[1].start, SOURCE.find("other").unwrap());
}

#[tokio::test]
async fn an_edit_that_belongs_to_no_symbol_stops_the_rename() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    // The second blank between `other` and `thing`: no identifier there.
    server.extra_edits = vec![(file.clone(), 2, 6, 7)];
    let candidate = candidate(&mut server, &file).await;

    let error = groups(&mut server, &Plain, &candidate, &file, "area")
        .await
        .err()
        .expect("a stray edit is refused")
        .to_string();

    assert!(
        error.contains("outside every place that refers to the symbol"),
        "{error}"
    );
    assert!(error.contains("line 3"), "{error}");
}
