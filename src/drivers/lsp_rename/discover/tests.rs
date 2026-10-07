use super::*;
use crate::drivers::lsp_rename::test_language::{Plain, WordServer};
use crate::drivers::symbol_scan;

const SOURCE: &str = "def area\ncall area\n";

fn project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.pl");
    std::fs::write(&file, SOURCE).unwrap();
    (dir, file)
}

fn occurrences(symbol: &str) -> Vec<Occurrence> {
    symbol_scan::occurrences(
        SOURCE,
        symbol,
        None,
        None,
        |c| c == '_' || c.is_alphanumeric(),
        symbol_scan::line_column(SOURCE),
    )
    .unwrap()
}

#[tokio::test]
async fn the_server_reason_for_refusing_a_known_symbol_is_the_answer() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    server.refusal = Some("a method named surface already exists".to_string());

    let error = discover(
        &mut server,
        &Plain,
        &file,
        SOURCE,
        &occurrences("area"),
        "area",
        "surface",
    )
    .await
    .err()
    .expect("the server refuses")
    .to_string();

    assert_eq!(
        error,
        "Cannot rename: a method named surface already exists"
    );
}

#[tokio::test]
async fn one_symbol_with_several_occurrences_is_one_candidate_with_all_references() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);

    let candidate = discover(
        &mut server,
        &Plain,
        &file,
        SOURCE,
        &occurrences("area"),
        "area",
        "surface",
    )
    .await
    .unwrap();

    // The second occurrence lies inside the first one's references: the same
    // symbol, asked about once.
    assert_eq!(server.renames_asked, 1);
    assert_eq!(candidate.references.len(), 2);
    assert_eq!(candidate.anchor_start, 4);
    assert_eq!(candidate.plan.edit_count(), 2);
}

#[tokio::test]
async fn the_edits_of_a_rename_are_not_written_by_planning() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);

    discover(
        &mut server,
        &Plain,
        &file,
        SOURCE,
        &occurrences("area"),
        "area",
        "surface",
    )
    .await
    .unwrap();

    assert_eq!(std::fs::read_to_string(&file).unwrap(), SOURCE);
}
