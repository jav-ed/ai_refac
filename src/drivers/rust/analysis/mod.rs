//! What the crate looks like to rust-analyzer: the loaded Cargo workspace
//! (`workspace`) and the modules in it with the files and declarations they
//! live in (`module_graph`), and the lock file a load may leave behind
//! (`lockfile`).

pub(super) mod lockfile;
pub(super) mod module_graph;
pub(super) mod workspace;
