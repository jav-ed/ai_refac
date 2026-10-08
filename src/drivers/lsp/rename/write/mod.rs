//! Writing a verified rename: `apply` writes the files, `journal` keeps the
//! undo log so a failure puts every file back.

pub mod apply;
pub mod journal;
