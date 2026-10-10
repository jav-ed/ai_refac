//! One Kotlin server for many tests. A Kotlin test used to start its own
//! server and import the Gradle project: 40 seconds before the first request,
//! whatever the test asked. Here the server is started once for a fixture,
//! kept for the whole test program, and handed to one test at a time (a test
//! that waits for the lease waits for the one before it). Between two tests
//! the project directory is put back as the fixture was, and the server is
//! told what changed, so a test starts from a project that equals a fresh
//! copy. A different fixture stops the server and starts another.
//!
//! What a test can no longer rely on is a clean server: after a failed
//! operation the server holds texts and file events that the rolled-back disk
//! no longer has, and the lease tells it again what the disk has for every
//! file it was shown (`recover`). A test that is about the server's own start and stop (the CLI dispatch, the
//! dry-run plans, `server.rs`) does not use a lease.
//!
//! The server and the project directory are removed when the test program
//! ends (`atexit`): nothing a test starts may outlive it.

use super::disk::{self, Files};
use refac::drivers::kotlin::moves::{MoveReport, move_files_on};
use refac::drivers::kotlin::rename::{
    RenameReport, RenameRequest, SharedServer, rename_all_symbols, rename_all_symbols_on,
    rename_symbol, rename_symbol_on,
};
use refac::drivers::kotlin::resync::{DiskChanges, follow_disk};
use refac::drivers::kotlin::server::{self, KotlinServer};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, LazyLock};
use tempfile::TempDir;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Mutex, OwnedMutexGuard};

/// The server's child process and its pipes belong to the runtime that
/// started it, and a `#[tokio::test]` runtime ends with its test. This one
/// lives as long as the program.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("the runtime of the Kotlin server")
});

static POOL: LazyLock<Arc<Mutex<Option<Held>>>> = LazyLock::new(|| {
    // SAFETY: `atexit` takes a plain function; it is registered once.
    unsafe { atexit(stop_at_exit) };
    Arc::new(Mutex::new(None))
});

unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> std::ffi::c_int;
}

extern "C" fn stop_at_exit() {
    RUNTIME.block_on(async {
        if let Some(held) = POOL.lock().await.take() {
            held.stop().await;
        }
    });
}

/// How long a Gradle daemon of the compile checks may sit idle if the program
/// is killed before it can stop it.
const COMPILE_DAEMON_IDLE_MS: u64 = 180_000;

struct Held {
    fixture: String,
    dir: TempDir,
    baseline: Files,
    directories: BTreeSet<String>,
    /// What the server has been told about the files.
    known: Files,
    /// None only while a server is being replaced.
    server: Option<SharedServer>,
    /// The server may hold texts that are not on disk: do not reuse it.
    uncertain: bool,
    /// A Gradle daemon was started for the compile checks.
    compiled: bool,
}

impl Held {
    async fn start(fixture: &str) -> Self {
        let dir = super::setup_fixture(fixture);
        let baseline = super::kotlin::snapshot(dir.path());
        let directories = disk::directories(dir.path());
        let server = boot(dir.path()).await;
        Self {
            fixture: fixture.to_string(),
            known: baseline.clone(),
            baseline,
            directories,
            dir,
            server: Some(server),
            uncertain: false,
            compiled: false,
        }
    }

    /// The server holds texts and file events that the disk no longer has: a
    /// failed operation was rolled back on disk without telling it. Tell it
    /// again what the disk has for every file it was ever shown, so that it
    /// can be trusted like a new one without paying for a new one.
    async fn recover(&mut self) {
        eprintln!(
            "[pool] {}: telling the server what the disk has",
            self.fixture
        );
        let server = self.server();
        let mut guard = server.lock().await;
        let mut changes = DiskChanges::default();
        for path in guard.told() {
            if path.is_file() {
                changes.created.push(path);
            } else {
                changes.deleted.push(path);
            }
        }
        follow_disk(&mut guard, self.dir.path(), &changes)
            .await
            .unwrap_or_else(|error| {
                panic!("the Kotlin server could not follow the disk: {error:#}")
            });
        self.uncertain = false;
    }

    fn server(&self) -> SharedServer {
        self.server.clone().expect("the lease has a server")
    }

    /// Tell the server what differs between what it was told and the disk.
    async fn tell_server(&mut self) {
        let root = self.dir.path();
        let now = super::kotlin::snapshot(root);
        let changes = disk::changes(root, &self.known, &now);
        follow_disk(&mut *self.server().lock().await, root, &changes)
            .await
            .unwrap_or_else(|error| {
                panic!("the Kotlin server could not follow the disk: {error:#}")
            });
        self.known = now;
    }

    /// Ready for the next operation: a server that can be trusted, told of
    /// whatever the test changed on disk itself.
    async fn prepare(&mut self) {
        if self.uncertain {
            self.recover().await;
        }
        self.tell_server().await;
    }

    /// The operation is over. `sent` is what the server had been sent before
    /// it. A server that was not involved in a failure is as good as before.
    async fn finish(&mut self, succeeded: bool, sent: u64) {
        let touched = self.server().lock().await.sent() != sent;
        if succeeded || !touched {
            self.known = super::kotlin::snapshot(self.dir.path());
            self.uncertain = false;
        } else {
            self.uncertain = true;
        }
    }

    async fn stop(mut self) {
        if let Some(server) = self.server.take() {
            stop_server(server).await;
        }
        if self.compiled {
            let _ = Command::new("./gradlew")
                .args(["--stop", "--console=plain", "-q"])
                .current_dir(self.dir.path())
                .output();
        }
    }
}

async fn boot(root: &Path) -> SharedServer {
    let install = server::locate().unwrap_or_else(|error| panic!("{error:#}"));
    let root = root.to_path_buf();
    let began = std::time::Instant::now();
    let started = RUNTIME
        .spawn(async move { KotlinServer::start(&install, &root).await })
        .await
        .expect("the task that starts the Kotlin server");
    // Shown with `--nocapture`: what a start costs and how often one happens.
    eprintln!("[pool] Kotlin server started in {:?}", began.elapsed());
    Arc::new(Mutex::new(started.unwrap_or_else(|error| {
        panic!("the Kotlin server did not start: {error:#}")
    })))
}

async fn stop_server(shared: SharedServer) {
    // Nobody else holds it: the engine lets go of its share when it returns.
    let server = Arc::try_unwrap(shared)
        .unwrap_or_else(|_| panic!("the Kotlin server is still lent out"))
        .into_inner();
    RUNTIME
        .spawn(async move { server.shutdown().await })
        .await
        .expect("the task that stops the Kotlin server");
}

/// The right to use the server, and the project directory, for one test.
pub struct Lease {
    held: OwnedMutexGuard<Option<Held>>,
}

/// Wait for the server of `fixture` (started on first use) with the project
/// put back as the fixture was.
pub async fn lease(fixture: &str) -> Lease {
    super::kotlin::require_server();
    let mut guard = POOL.clone().lock_owned().await;
    match guard.take() {
        Some(mut held) if held.fixture == fixture => {
            eprintln!("[pool] {fixture}: reused");
            disk::restore(held.dir.path(), &held.baseline, &held.directories);
            held.prepare().await;
            *guard = Some(held);
        }
        Some(held) => {
            held.stop().await;
            *guard = Some(Held::start(fixture).await);
        }
        None => *guard = Some(Held::start(fixture).await),
    }
    Lease { held: guard }
}

impl Lease {
    fn held(&mut self) -> &mut Held {
        self.held.as_mut().expect("a lease holds a server")
    }

    pub fn path(&self) -> &Path {
        self.held
            .as_ref()
            .expect("a lease holds a server")
            .dir
            .path()
    }

    pub async fn move_files(&mut self, moves: &[(String, String)]) -> anyhow::Result<MoveReport> {
        let root = self.path().to_path_buf();
        let held = self.held();
        held.prepare().await;
        let server = held.server();
        let sent = server.lock().await.sent();
        held.uncertain = true;
        let outcome = move_files_on(&mut *server.lock().await, moves, &root).await;
        held.finish(outcome.is_ok(), sent).await;
        outcome
    }

    /// A rename in the leased project (the request's own `project_path` is
    /// replaced). A dry run plans on a server of its own and leaves the leased
    /// one alone: it would hold texts that never reach the disk.
    pub async fn rename(&mut self, mut request: RenameRequest) -> anyhow::Result<RenameReport> {
        request.project_path = self.path().to_path_buf();
        if request.dry_run {
            return rename_symbol(request).await;
        }
        let held = self.held();
        held.prepare().await;
        let server = held.server();
        let sent = server.lock().await.sent();
        held.uncertain = true;
        let outcome = rename_symbol_on(&server, request).await;
        held.finish(outcome.is_ok(), sent).await;
        outcome
    }

    /// Several renames in one session; see `rename`.
    pub async fn rename_all(
        &mut self,
        mut requests: Vec<RenameRequest>,
    ) -> anyhow::Result<Vec<RenameReport>> {
        for request in &mut requests {
            request.project_path = self.path().to_path_buf();
        }
        if requests.first().is_some_and(|request| request.dry_run) {
            return rename_all_symbols(requests).await;
        }
        let held = self.held();
        held.prepare().await;
        let server = held.server();
        let sent = server.lock().await.sent();
        held.uncertain = true;
        let outcome = rename_all_symbols_on(&server, requests).await;
        held.finish(outcome.is_ok(), sent).await;
        outcome
    }

    /// The judge of a refactor, as `kotlin::assert_compiles`, on a Gradle
    /// daemon that stays for the next test (it is stopped with the server).
    /// `--rerun-tasks` compiles everything again, so nothing a former test
    /// left behind can stand in for a file.
    pub fn assert_compiles(&mut self, tasks: &[&str]) {
        self.held().compiled = true;
        let idle = format!("-Dorg.gradle.daemon.idletimeout={COMPILE_DAEMON_IDLE_MS}");
        super::kotlin::compile(self.path(), tasks, &["--rerun-tasks", &idle]);
    }
}
