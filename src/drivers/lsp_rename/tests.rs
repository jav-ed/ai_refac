//! The engine's own decisions, against a server that only knows words
//! (`WordServer`): when an answer is asked for again, and when it is final.

use super::*;
use crate::drivers::symbol_rename::RenameRequest;
use test_language::{Plain, WordServer};

const SOURCE: &str = "def area\ncall area\ncall area\n";

/// Plain with a number of attempts.
struct Patient(usize);

#[async_trait::async_trait]
impl Language for Patient {
    fn name(&self) -> &'static str {
        Plain.name()
    }

    fn extensions(&self) -> &'static [&'static str] {
        Plain.extensions()
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        Plain.reserved_words()
    }

    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        Plain.project_root(project_path)
    }

    async fn start(&self, root: &Path, file: &Path) -> Result<Box<dyn RenameServer>> {
        Plain.start(root, file).await
    }

    fn rename_attempts(&self) -> usize {
        self.0
    }
}

/// The file exists on disk: the engine reads the edited files.
fn project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.pl");
    std::fs::write(&file, SOURCE).unwrap();
    (dir, file)
}

fn request(file: &Path) -> RenameRequest {
    RenameRequest {
        project_path: file.parent().unwrap().to_path_buf(),
        file: file.to_path_buf(),
        symbol: "area".to_string(),
        new_name: "surface".to_string(),
        line: None,
        column: None,
        dry_run: true,
    }
}

async fn plan(language: &dyn Language, server: &mut WordServer, file: &Path) -> Result<Candidate> {
    let request = request(file);
    let occurrences = symbol_scan::occurrences(
        SOURCE,
        "area",
        None,
        None,
        |c| c == '_' || c.is_alphanumeric(),
        symbol_scan::line_column(SOURCE),
    )?;
    let root = file.parent().unwrap();
    plan_and_verify(server, language, root, file, SOURCE, &occurrences, &request).await
}

#[tokio::test]
async fn a_partial_answer_is_asked_for_again_when_the_language_allows() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    server.partial_renames = 1;

    let candidate = plan(&Patient(3), &mut server, &file).await.unwrap();

    assert_eq!(candidate.plan.edit_count(), 3);
    assert_eq!(server.renames_asked, 2);
    // The retry planned against the original text, not a half-renamed one.
    assert_eq!(candidate.plan.files[0].file.before, SOURCE);
    assert_eq!(
        candidate.plan.files[0].file.text,
        "def surface\ncall surface\ncall surface\n"
    );
}

#[tokio::test]
async fn an_answer_that_stays_partial_ends_in_a_refusal_after_the_attempts() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    server.partial_renames = 10;

    let error = plan(&Patient(3), &mut server, &file)
        .await
        .err()
        .expect("the answer never becomes complete");

    assert!(
        error.to_string().contains("leaves 2 of the 3 places"),
        "{error:#}"
    );
    assert_eq!(server.renames_asked, 3);
}

#[tokio::test]
async fn a_language_with_one_attempt_takes_the_first_answer_as_final() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);
    server.partial_renames = 1;

    assert!(plan(&Plain, &mut server, &file).await.is_err());
    assert_eq!(server.renames_asked, 1);
}

#[tokio::test]
async fn a_complete_answer_is_taken_at_once() {
    let (_dir, file) = project();
    let mut server = WordServer::new(&[(&file, SOURCE)]);

    let candidate = plan(&Patient(4), &mut server, &file).await.unwrap();

    assert_eq!(candidate.plan.edit_count(), 3);
    assert_eq!(server.renames_asked, 1);
}
