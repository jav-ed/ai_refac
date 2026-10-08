//! What the crate looks like to rust-analyzer: the loaded Cargo workspace
//! (`workspace`) and the modules in it with the files and declarations they
//! live in (`module_graph`).

pub(super) mod module_graph;
pub(super) mod workspace;
