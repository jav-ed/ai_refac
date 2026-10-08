//! Carrying out a planned module move on disk and checking it afterwards:
//! `apply` writes the edits and moves the files with a rollback, `layout`
//! decides which files and folders make up a module, and `validation`
//! checks the paths before and the workspace after.

pub(super) mod apply;
pub(super) mod layout;
pub(super) mod validation;
