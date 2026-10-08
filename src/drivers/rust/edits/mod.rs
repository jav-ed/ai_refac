//! The edits of Rust source text that a module move needs: the new `mod` line
//! (`declarations`), the paths that name the module (`references`,
//! `macro_paths`, `imports`, `use_split`) and the `super::` paths inside the
//! moved files (`super_paths`).

pub(super) mod declarations;
pub(super) mod imports;
pub(super) mod macro_paths;
pub(super) mod references;
pub(super) mod super_paths;
pub(super) mod use_split;
