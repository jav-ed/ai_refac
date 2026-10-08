//! The text behind `refac --help` and `refac <command> --help`. `-h` shows the
//! one-line summary of a command; `--help` shows everything here: what the
//! command does, what it needs, what it writes, what it refuses, how to read
//! its output, and examples that can be pasted. One file per command, so a
//! change of behaviour has one obvious place to change its description.

pub(super) mod doctor;
pub(super) mod move_files;
pub(super) mod move_module;
pub(super) mod rename;
pub(super) mod top;
