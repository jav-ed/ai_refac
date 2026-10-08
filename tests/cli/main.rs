//! The command line itself: usage errors, help, doctor, and what `rename --batch` refuses before any server starts. Needs no language server.

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod batch_rename;
mod doctor;
mod help;
mod move_outcome;
mod usage;
