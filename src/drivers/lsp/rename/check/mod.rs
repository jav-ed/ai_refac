//! Proving a planned rename before anything is written: `verify` compares the
//! reference sites before and after in memory, `leftovers` looks for the old
//! name where the server did not edit.

pub(super) mod leftovers;
pub(super) mod verify;
