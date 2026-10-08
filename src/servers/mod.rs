//! The language servers refac starts, and how it finds them.
//!
//! refac never leaves a server running: it starts one for a command and stops
//! it when the command is over, because an idle language server holds
//! hundreds of megabytes of RAM. So a server only has to be installed. This
//! module knows where each one may be installed (`catalog/`), searches those
//! places in a fixed order, and when it finds nothing says exactly where it
//! looked and which command (`refac doctor <language>`) teaches the install.

mod catalog;
mod doctor;
mod handshake;
mod locate;

use crate::drivers::lsp::client::Server as Profile;
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub use catalog::{all, for_language};
pub use doctor::{
    Report, Status, doctor_all, doctor_one, render_json, render_one, render_overview,
};
pub use handshake::StartCheck;
pub use locate::{Attempt, Located, NotFound, Outcome, locate, locate_with};

/// A language server refac can start for a language.
pub struct Server {
    /// What `refac doctor <language>` accepts; the first is the canonical name.
    pub languages: &'static [&'static str],
    /// "Go", for messages.
    pub language: &'static str,
    /// "gopls", for messages.
    pub name: &'static str,
    /// What refac uses it for, in one sentence.
    pub used_for: &'static str,
    /// An environment variable that names the server (or its folder) and
    /// beats every other place.
    pub env_var: &'static str,
    pub find: Find,
    /// Other tools the server needs to work.
    pub requirements: &'static [Requirement],
    /// Numbered steps that install the server, copy-pasteable.
    pub install: &'static [&'static str],
}

pub enum Find {
    /// An executable on PATH, or in folders where installers put it.
    Executable {
        candidates: &'static [Candidate],
        /// Folders to look in after PATH, given the project directory.
        folders: fn(&Path) -> Vec<PathBuf>,
        /// How refac starts it, and the protocol profile it speaks.
        launch: Launch,
    },
    /// A downloaded folder that the environment variable points to (the
    /// Kotlin server).
    Folder {
        /// The executable, relative to the folder.
        executable: &'static str,
        /// A file that names the build and proves it is the right download.
        build_file: &'static str,
    },
    /// Shipped with refac and installed by it on first use (the TypeScript
    /// engine); there is nothing to search.
    Bundled { check: fn() -> Vec<Attempt> },
}

/// One executable name to look for and how to prove it runs.
pub struct Candidate {
    pub executable: &'static str,
    /// Arguments that make it print its version and exit. A server that does
    /// not answer `--version` is proved by a companion command instead.
    pub version: Version,
}

pub enum Version {
    /// `<executable> <args>`
    Own(&'static [&'static str]),
    /// `<sibling> <args>`, a command installed with the server.
    Companion(&'static str, &'static [&'static str]),
}

pub struct Launch {
    pub profile: Profile,
    pub args: &'static [&'static str],
}

pub struct Requirement {
    pub name: &'static str,
    pub executable: &'static str,
    pub why: &'static str,
    pub hint: &'static str,
}

/// The working executable of the server for `language`, or an error that
/// says where it was looked for and which command teaches the install.
pub fn executable(language: &str, project: &Path) -> Result<PathBuf> {
    let server = for_language(language)?;
    Ok(locate(server, project)?.executable)
}

/// The message for a command that needs the server of `language` and cannot
/// find a working one, or `None` when it is fine (or the language has no
/// server to look for).
pub fn missing_server_message(language: &str, project: &Path) -> Option<String> {
    let server = for_language(language).ok()?;
    match locate(server, project) {
        Ok(_) => None,
        Err(not_found) => Some(not_found.to_string()),
    }
}

/// `refac doctor` takes a language name; this turns a typo into the list of
/// names that work.
pub(crate) fn unknown_language(name: &str) -> Result<&'static Server> {
    let known: Vec<&str> = all().iter().map(|server| server.languages[0]).collect();
    bail!(
        "`{name}` is not a language refac starts a server for. Known: {}",
        known.join(", ")
    )
}
