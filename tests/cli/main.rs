//! The command line itself: usage errors, help, doctor, and what `move --batch` and `rename --batch` refuse before any server starts. Needs no language server.

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod batch_move;
mod batch_rename;
mod doctor;
mod help;
mod kotlin_cost;
mod layout;
mod move_outcome;
mod usage;
