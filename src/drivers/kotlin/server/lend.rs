//! A server somebody else keeps. A command starts a server and stops it again,
//! which is right for a command and costs half a minute of Gradle import. A
//! caller that runs many operations against one project in one process (the
//! tests of this repository start one server per fixture) lends the server it
//! keeps for a Gradle root, and the move and rename entry points use it for
//! that root instead of starting their own. No command lends a server, and a
//! root nobody lent one for behaves as it always did.
//!
//! The keeper is responsible for the server: it stops it, and before it
//! lends it to the next operation it tells it what the files on disk are
//! (`resync::follow_disk`), because an operation that failed was rolled back
//! on disk without telling the server.

use super::KotlinServer;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A running server shared between its keeper and whoever it is lent to.
pub type SharedServer = Arc<tokio::sync::Mutex<KotlinServer>>;

static LENT: Mutex<Vec<(PathBuf, SharedServer)>> = Mutex::new(Vec::new());

/// A path as the lent list knows it. A path that does not exist matches
/// nothing that was lent, because a lent root exists.
fn key(root: &Path) -> PathBuf {
    std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf())
}

/// Use `server` for every move and rename in the Gradle root `root` until it
/// is recalled.
pub fn lend(root: &Path, server: SharedServer) {
    let root = key(root);
    let mut lent = LENT.lock().expect("the list of lent servers");
    lent.retain(|(lent_root, _)| lent_root != &root);
    lent.push((root, server));
}

/// The server is no longer lent for `root`.
pub fn recall(root: &Path) {
    let root = key(root);
    LENT.lock()
        .expect("the list of lent servers")
        .retain(|(lent_root, _)| lent_root != &root);
}

/// The server lent for `root`, if there is one.
pub fn lent_for(root: &Path) -> Option<SharedServer> {
    let root = key(root);
    LENT.lock()
        .expect("the list of lent servers")
        .iter()
        .find(|(lent_root, _)| lent_root == &root)
        .map(|(_, server)| server.clone())
}
