//! Planning a rename without writing: find the symbol (`discover`), the
//! related symbols that must change with it (`related`, `family`), the names
//! that are allowed (`names`), the edits the server answers with (`edits`) and
//! the comment lines that follow the new name (`comments`).

pub mod comments;
pub mod discover;
pub mod edits;
pub(super) mod family;
pub(super) mod names;
pub(super) mod related;
