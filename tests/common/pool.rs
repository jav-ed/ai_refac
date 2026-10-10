//! One Kotlin server for many tests. A Kotlin test used to start its own
//! server and import the Gradle project: 20 to 47 seconds before the first request,
//! whatever the test asked. Here a server is started once for a fixture, kept
//! for the whole test program, and lent (`server::lend`) for the project
//! directory of the fixture, so that the ordinary entry points of the library
//! (`move_files`, `rename_symbol`, `handle_refactor`) use it instead of
//! starting their own. The tests call those entry points as they always did.
//!
//! A test holds a lease on the pool, and a test that waits for the lease waits
//! for the one before it. When a test takes the lease the project directory is
//! put back as the fixture was and the server is told what the disk has for
//! every file it was ever shown (`recover`), so a test starts from a project
//! that equals a fresh copy and a server that agrees with it, whatever the
//! test before it did, a failure included. The servers of all fixtures that
//! were used stay alive (about 2 GB each); each stops when the test program
//! ends.
//!
//! A test that is about the server's own start and stop (`server.rs`, the
//! dry-run plans that run the `refac` binary) does not lease: it starts what
//! it needs.
//!
//! The servers and the project directories are removed when the test program
//! ends (`atexit`): nothing a test starts may outlive it.

use super::disk;
use refac::drivers::kotlin::resync::{DiskChanges, follow_disk};
use refac::drivers::kotlin::server::{self, KotlinServer, SharedServer};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
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

#[derive(Default)]
struct Pool {
    held: HashMap<String, Held>,
}

static POOL: LazyLock<Arc<Mutex<Pool>>> = LazyLock::new(|| {
    // SAFETY: `atexit` takes a plain function; it is registered once.
    unsafe { atexit(stop_at_exit) };
    Arc::new(Mutex::new(Pool::default()))
});

unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> std::ffi::c_int;
}

extern "C" fn stop_at_exit() {
    RUNTIME.block_on(async {
        let held = std::mem::take(&mut POOL.lock().await.held);
        for (_, held) in held {
            held.stop().await;
        }
    });
}

/// How long a Gradle daemon of the compile checks may sit idle if the program
/// is killed before it can stop it.
const COMPILE_DAEMON_IDLE_MS: u64 = 180_000;

struct Held {
    dir: TempDir,
    baseline: disk::Files,
    directories: BTreeSet<String>,
    server: SharedServer,
    /// A Gradle daemon was started for the compile checks.
    compiled: AtomicBool,
}

impl Held {
    async fn start(fixture: &str) -> Self {
        let dir = super::setup_fixture(fixture);
        let baseline = super::kotlin::snapshot(dir.path());
        let directories = disk::directories(dir.path());
        let server = boot(dir.path()).await;
        server::lend(dir.path(), server.clone());
        Self {
            baseline,
            directories,
            dir,
            server,
            compiled: AtomicBool::new(false),
        }
    }

    /// The project as the fixture was, and a server that agrees with the disk.
    async fn reset(&mut self) {
        disk::restore(self.dir.path(), &self.baseline, &self.directories);
        self.recover().await;
    }

    /// Tell the server what the disk has for every file it was ever shown: a
    /// failed operation was rolled back on disk without telling it, and the
    /// operation before may have left documents open with texts that never
    /// reached the disk. Every open document is closed, and a file that
    /// exists is announced as new (the server reads it again), one that is
    /// gone as deleted.
    async fn recover(&mut self) {
        let mut server = self.server.lock().await;
        let mut changes = DiskChanges::default();
        for path in server.told() {
            if path.is_file() {
                changes.created.push(path);
            } else {
                changes.deleted.push(path);
            }
        }
        follow_disk(&mut server, self.dir.path(), &changes)
            .await
            .unwrap_or_else(|error| {
                panic!("the Kotlin server could not follow the disk: {error:#}")
            });
    }

    async fn stop(self) {
        server::recall(self.dir.path());
        // Nobody else holds it: an operation lets go of its share when it ends.
        let server = Arc::try_unwrap(self.server)
            .unwrap_or_else(|_| panic!("the Kotlin server is still in use"))
            .into_inner();
        RUNTIME
            .spawn(async move { server.shutdown().await })
            .await
            .expect("the task that stops the Kotlin server");
        if self.compiled.load(Ordering::Relaxed) {
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

/// The right to use the pool, and the project directory of one fixture, for
/// one test.
pub struct Lease {
    pool: OwnedMutexGuard<Pool>,
    fixture: String,
}

/// Wait for the server of `fixture` (started on first use), with the project
/// put back as the fixture was.
pub async fn lease(fixture: &str) -> Lease {
    super::kotlin::require_server();
    let mut pool = POOL.clone().lock_owned().await;
    match pool.held.get_mut(fixture) {
        Some(held) => {
            eprintln!("[pool] {fixture}: reused");
            held.reset().await;
        }
        None => {
            let held = Held::start(fixture).await;
            pool.held.insert(fixture.to_string(), held);
        }
    }
    Lease {
        pool,
        fixture: fixture.to_string(),
    }
}

impl Lease {
    fn held(&mut self) -> &mut Held {
        self.pool
            .held
            .get_mut(&self.fixture)
            .expect("a lease holds a server")
    }

    /// The project directory: the Gradle root the server is lent for.
    pub fn path(&self) -> &Path {
        self.pool.held[&self.fixture].dir.path()
    }

    /// A test that edited sources itself, after it took the lease, tells the
    /// server before the next operation.
    pub async fn sync(&mut self) {
        self.held().recover().await;
    }

    /// The judge of a refactor, as `kotlin::assert_compiles`, on a Gradle
    /// daemon that stays for the next test (it is stopped with the server).
    /// `--rerun-tasks` compiles everything again, so nothing a former test
    /// left behind can stand in for a file.
    pub fn assert_compiles(&self, tasks: &[&str]) {
        self.pool.held[&self.fixture]
            .compiled
            .store(true, Ordering::Relaxed);
        let idle = format!("-Dorg.gradle.daemon.idletimeout={COMPILE_DAEMON_IDLE_MS}");
        super::kotlin::compile(self.path(), tasks, &["--rerun-tasks", &idle]);
    }
}
