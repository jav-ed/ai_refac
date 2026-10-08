//! A batch against a server that only knows words (`WordServer`) started from
//! the `.pl` files on disk: one server for all renames, each rename sees what
//! the previous one wrote, a failure takes the earlier renames back, and
//! nothing starts when the requests can be refused beforehand.

use super::*;
use crate::drivers::lsp::rename::language::Language;
use crate::drivers::lsp::rename::test_language::WordServer;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Plain words as a language, with a server per `start` that holds the files
/// of the project as they are on disk, and counters for the starts and stops.
#[derive(Default)]
struct Disk {
    starts: Arc<AtomicUsize>,
    stops: Arc<AtomicUsize>,
}

/// A `WordServer` that counts its stop.
struct Counted {
    inner: WordServer,
    stops: Arc<AtomicUsize>,
}

#[async_trait]
impl RenameServer for Counted {
    async fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.inner.request(method, params).await
    }

    async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        self.inner.sync_document(path, text).await
    }

    async fn shutdown(self: Box<Self>) {
        self.stops.fetch_add(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl Language for Disk {
    fn name(&self) -> &'static str {
        "Plain"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["pl"]
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        &["class", "def"]
    }

    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        Ok(project_path.to_path_buf())
    }

    async fn start(&self, root: &Path, _file: &Path) -> Result<Box<dyn RenameServer>> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        let files: Vec<(PathBuf, String)> = std::fs::read_dir(root)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .filter(|path| path.extension().is_some_and(|extension| extension == "pl"))
            .map(|path| std::fs::read_to_string(&path).map(|text| (path, text)))
            .collect::<std::io::Result<_>>()?;
        let shown: Vec<(&Path, &str)> = files
            .iter()
            .map(|(path, text)| (path.as_path(), text.as_str()))
            .collect();
        Ok(Box::new(Counted {
            inner: WordServer::new(&shown),
            stops: self.stops.clone(),
        }))
    }
}

impl Disk {
    fn starts(&self) -> usize {
        self.starts.load(Ordering::SeqCst)
    }

    fn stops(&self) -> usize {
        self.stops.load(Ordering::SeqCst)
    }
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, text) in files {
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    dir
}

fn request(dir: &Path, file: &str, symbol: &str, new_name: &str, dry_run: bool) -> RenameRequest {
    RenameRequest {
        project_path: dir.to_path_buf(),
        file: PathBuf::from(file),
        symbol: symbol.to_string(),
        new_name: new_name.to_string(),
        line: None,
        column: None,
        dry_run,
    }
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
}

const DECLARATIONS: &str = "def area\ndef width\n";
const USES: &str = "call area\ncall width\ncall area\n";

#[tokio::test]
async fn several_renames_share_one_server_and_one_stop() {
    let dir = project(&[("a.pl", DECLARATIONS), ("b.pl", USES)]);
    let language = Disk::default();

    let reports = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", false),
            request(dir.path(), "a.pl", "width", "breadth", false),
        ],
    )
    .await
    .unwrap();

    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].edits, 3);
    assert_eq!(reports[1].edits, 2);
    assert_eq!(read(dir.path(), "a.pl"), "def surface\ndef breadth\n");
    assert_eq!(
        read(dir.path(), "b.pl"),
        "call surface\ncall breadth\ncall surface\n"
    );
    assert_eq!((language.starts(), language.stops()), (1, 1));
}

#[tokio::test]
async fn a_rename_sees_what_the_one_before_it_wrote() {
    let dir = project(&[("a.pl", DECLARATIONS), ("b.pl", USES)]);
    let language = Disk::default();

    // The second rename names the symbol by the name the first gave it.
    let reports = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", false),
            request(dir.path(), "a.pl", "surface", "extent", false),
        ],
    )
    .await
    .unwrap();

    assert_eq!(reports[1].edits, 3);
    assert_eq!(read(dir.path(), "a.pl"), "def extent\ndef width\n");
    assert_eq!(
        read(dir.path(), "b.pl"),
        "call extent\ncall width\ncall extent\n"
    );
}

#[tokio::test]
async fn a_failing_rename_takes_the_earlier_ones_back() {
    let dir = project(&[("a.pl", DECLARATIONS), ("b.pl", USES)]);
    let language = Disk::default();

    let error = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", false),
            request(dir.path(), "a.pl", "volume", "capacity", false),
        ],
    )
    .await
    .unwrap_err();

    let message = format!("{error:#}");
    assert!(
        message.contains("Rename 2 of 2 (volume -> capacity"),
        "{message}"
    );
    assert!(
        message.contains("the 1 earlier rename(s) were undone, so nothing was changed"),
        "{message}"
    );
    assert_eq!(read(dir.path(), "a.pl"), DECLARATIONS);
    assert_eq!(read(dir.path(), "b.pl"), USES);
    // The server is stopped on the failure too.
    assert_eq!((language.starts(), language.stops()), (1, 1));
}

#[tokio::test]
async fn dry_run_renames_are_planned_against_the_files_as_they_are() {
    let dir = project(&[("a.pl", DECLARATIONS), ("b.pl", USES)]);
    let language = Disk::default();

    // Both rename `area`. The second must not see the first one's proof text
    // in b.pl, or it would find fewer places than there are.
    let reports = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", true),
            request(dir.path(), "a.pl", "area", "extent", true),
        ],
    )
    .await
    .unwrap();

    assert!(reports.iter().all(|report| report.dry_run));
    assert_eq!(reports[0].edits, 3);
    assert_eq!(reports[1].files.len(), reports[0].files.len());
    assert_eq!(reports[1].edits, 3);
    assert_eq!(read(dir.path(), "a.pl"), DECLARATIONS);
    assert_eq!(read(dir.path(), "b.pl"), USES);
    assert_eq!((language.starts(), language.stops()), (1, 1));
}

#[tokio::test]
async fn a_request_that_cannot_work_starts_no_server() {
    let dir = project(&[("a.pl", DECLARATIONS), ("b.pl", USES)]);
    let language = Disk::default();

    let keyword = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", false),
            request(dir.path(), "a.pl", "width", "class", false),
        ],
    )
    .await
    .unwrap_err();
    assert!(
        format!("{keyword:#}").contains("Rename 2 of 2 (width -> class"),
        "{keyword:#}"
    );
    assert!(
        format!("{keyword:#}").contains("is a Plain keyword"),
        "{keyword:#}"
    );

    let mixed = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", false),
            request(dir.path(), "a.pl", "width", "breadth", true),
        ],
    )
    .await
    .unwrap_err();
    assert!(
        format!("{mixed:#}").contains("every rename of a batch is a dry run or none is"),
        "{mixed:#}"
    );

    let other = project(&[]);
    let apart = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "area", "surface", false),
            request(other.path(), "a.pl", "width", "breadth", false),
        ],
    )
    .await
    .unwrap_err();
    assert!(
        format!("{apart:#}").contains("a batch works in one project"),
        "{apart:#}"
    );

    let empty = rename_all(&language, Vec::new()).await.unwrap_err();
    assert!(
        format!("{empty:#}").contains("at least one rename"),
        "{empty:#}"
    );

    // The first rename is looked up before the server starts as well.
    let missing = rename_all(
        &language,
        vec![
            request(dir.path(), "a.pl", "volume", "capacity", false),
            request(dir.path(), "a.pl", "width", "breadth", false),
        ],
    )
    .await
    .unwrap_err();
    assert!(
        format!("{missing:#}").contains("Rename 1 of 2 (volume -> capacity"),
        "{missing:#}"
    );

    assert_eq!((language.starts(), language.stops()), (0, 0));
    assert_eq!(read(dir.path(), "a.pl"), DECLARATIONS);
}

#[tokio::test]
async fn a_single_rename_keeps_its_plain_error() {
    let dir = project(&[("a.pl", DECLARATIONS)]);
    let language = Disk::default();

    let error = rename_all(
        &language,
        vec![request(dir.path(), "a.pl", "width", "class", false)],
    )
    .await
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "`class` is a Plain keyword and cannot name a symbol"
    );
}
