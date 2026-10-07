//! What a language adds to the rename engine: how its identifiers look, which
//! server answers for it, and the few places where that server behaves in a
//! way the engine has to expect.

use super::discover::RenamePlan;
use super::journal::FileWrite;
use super::server::RenameServer;
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// What a rename implies beyond the server's own edits: files refac writes
/// itself (Android XML) and things the user should know.
#[derive(Default)]
pub struct FollowUps {
    pub writes: Vec<FileWrite>,
    pub notes: Vec<String>,
}

/// A byte range of a source file that the language server does not rename.
pub struct UnrenamedPlace {
    pub start: usize,
    pub end: usize,
    /// What the place is, for the note: "a macro_rules! definition".
    pub what: &'static str,
}

/// The edits one file gets, for a language to tell which of them it expects
/// outside every reference of the symbol.
pub struct EditedText<'a> {
    pub before: &'a str,
    /// Byte range of each edit in `before`.
    pub spans: Vec<(usize, usize)>,
    /// What each edit puts there, in the same order.
    pub new_texts: Vec<&'a str>,
    pub symbol: &'a str,
}

#[async_trait]
pub trait Language: Sync {
    /// For messages: "Go", "Rust", "Python", "Kotlin".
    fn name(&self) -> &'static str;

    /// File extensions (without the dot) the language's sources have.
    fn extensions(&self) -> &'static [&'static str];

    fn is_identifier_char(&self, character: char) -> bool {
        character == '_' || character.is_alphanumeric()
    }

    /// Words that cannot name a symbol.
    fn reserved_words(&self) -> &'static [&'static str];

    /// The directory the server is started on, from `--project-path`. Fails
    /// when the path is not a project of this language.
    fn project_root(&self, project_path: &Path) -> Result<PathBuf>;

    /// Names the language itself gives a meaning to by their spelling (Python
    /// `__init__`), with the reason the rename is refused. Checked before any
    /// server starts.
    fn refuse_names(&self, _symbol: &str, _new_name: &str) -> Option<String> {
        None
    }

    /// Start the server on the project and return once it can answer.
    async fn start(&self, root: &Path, file: &Path) -> Result<Box<dyn RenameServer>>;

    /// Whether the server renames a method without its overrides (basedpyright
    /// does), so the engine asks it for the overrides and renames them too.
    fn renames_overrides(&self) -> bool {
        false
    }

    /// What to tell the user whose name the server says is not a symbol, when
    /// the language has a common cause: the name of a module in an import is
    /// a file, which `refac move` renames.
    fn not_a_symbol_hint(&self) -> Option<&'static str> {
        None
    }

    /// Which edits of one file the language expects outside every reference
    /// of the symbol: the symbol in a doc comment, an import line the server
    /// does not list as a reference. Every other edit outside a reference
    /// stops the rename.
    fn exempt_edits(&self, edited: &EditedText) -> Vec<bool> {
        vec![false; edited.spans.len()]
    }

    /// How often the rename is asked again when the server's answer fails the
    /// proof. A server that gives up part of a rename under load (gopls) gets
    /// more than one try; a refusal that holds every time is the real answer.
    fn rename_attempts(&self) -> usize {
        1
    }

    /// Parts of `text` where the server is known not to rename the symbol
    /// even though it is used there (the body of a Rust `macro_rules!`): byte
    /// ranges with the reason. A leftover inside one is reported as an
    /// attention note, because the program then no longer builds.
    fn unrenamed_places(&self, _text: &str) -> Vec<UnrenamedPlace> {
        Vec::new()
    }

    /// `None` when renaming may move files along with the symbol (a Kotlin
    /// class and its file). Otherwise what to tell the user who asked for a
    /// rename the server answers with file operations.
    fn refuse_file_operations(&self) -> Option<&'static str> {
        Some("refac rename edits symbols inside files; to rename or move files use `refac move`")
    }

    /// Changes beyond the server's edits, planned without writing.
    fn follow_ups(&self, _root: &Path, _plan: &RenamePlan) -> Result<FollowUps> {
        Ok(FollowUps::default())
    }
}
