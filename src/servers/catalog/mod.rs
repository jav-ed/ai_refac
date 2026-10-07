//! One file per server: where it can be installed, what it needs, and the
//! steps that install it. Update the steps here when an installer changes.

mod dart;
mod go;
mod kotlin;
mod python;
mod rust;
mod typescript;

use super::{Server, unknown_language};
use anyhow::Result;

static ALL: [&Server; 6] = [
    &go::GO,
    &rust::RUST,
    &python::PYTHON,
    &dart::DART,
    &kotlin::KOTLIN,
    &typescript::TYPESCRIPT,
];

pub fn all() -> &'static [&'static Server] {
    &ALL
}

/// The server for a language name such as `go`, `rust`, `python`, `dart`,
/// `kotlin`, `typescript` (also `golang`, `rs`, `py`, `ts`, `js`, ...).
pub fn for_language(name: &str) -> Result<&'static Server> {
    let wanted = name.to_ascii_lowercase();
    all()
        .iter()
        .copied()
        .find(|server| server.languages.contains(&wanted.as_str()))
        .map_or_else(|| unknown_language(name), Ok)
}

/// `~/<relative>` when HOME is set.
pub(super) fn in_home(relative: &str) -> Vec<std::path::PathBuf> {
    std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join(relative))
        .into_iter()
        .collect()
}
