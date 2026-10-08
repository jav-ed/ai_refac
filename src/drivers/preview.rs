//! What a `move --dry-run` reports for one language: the paths that would move,
//! the edits per file, and the notes the real move would print. A driver plans
//! it without writing a file (or on a copy of the project that is thrown away),
//! so the preview and the real move are the same engine.

use std::collections::BTreeMap;
use std::path::PathBuf;

pub mod copy;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MovePreview {
    /// Where each requested path goes, as the driver resolved it.
    pub moves: Vec<(PathBuf, PathBuf)>,
    /// Edits per file, keyed by the path the file has now (before the move).
    /// An edit is one rewritten reference: an import, a link, a path.
    pub edits: BTreeMap<PathBuf, usize>,
    /// What the real move would say besides success.
    pub notes: Vec<String>,
}

impl MovePreview {
    /// Adds `count` edits to `path`; zero edits are not recorded.
    pub fn add_edits(&mut self, path: PathBuf, count: usize) {
        if count > 0 {
            *self.edits.entry(path).or_insert(0) += count;
        }
    }

    pub fn total_edits(&self) -> usize {
        self.edits.values().sum()
    }

    /// Adds the file renames a language server put in its plan when the
    /// requested move does not already list the file: a server that moves a
    /// package or a module moves more than was asked for.
    pub fn add_server_renames(&mut self, renames: &[(PathBuf, PathBuf)]) {
        for (from, to) in renames {
            if !self.moves.iter().any(|(known, _)| known == from) {
                self.moves.push((from.clone(), to.clone()));
            }
        }
    }

    /// Joins the preview of another language into this one.
    pub fn absorb(&mut self, other: MovePreview) {
        self.moves.extend(other.moves);
        for (path, count) in other.edits {
            self.add_edits(path, count);
        }
        self.notes.extend(other.notes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_add_up_per_file_and_zero_is_not_recorded() {
        let mut preview = MovePreview::default();
        preview.add_edits("a.ts".into(), 2);
        preview.add_edits("a.ts".into(), 1);
        preview.add_edits("b.ts".into(), 0);
        assert_eq!(preview.edits.len(), 1);
        assert_eq!(preview.edits[&PathBuf::from("a.ts")], 3);
        assert_eq!(preview.total_edits(), 3);
    }

    #[test]
    fn a_rename_the_server_adds_is_listed_once() {
        let mut preview = MovePreview {
            moves: vec![("a.rs".into(), "b.rs".into())],
            ..Default::default()
        };
        preview.add_server_renames(&[
            ("a.rs".into(), "b.rs".into()),
            ("a/x.rs".into(), "b/x.rs".into()),
        ]);
        assert_eq!(preview.moves.len(), 2);
        assert_eq!(preview.moves[1].0, PathBuf::from("a/x.rs"));
    }

    #[test]
    fn absorb_keeps_moves_edits_and_notes() {
        let mut first = MovePreview::default();
        first.add_edits("a.ts".into(), 1);
        let mut second = MovePreview {
            moves: vec![("x".into(), "y".into())],
            notes: vec!["n".into()],
            ..Default::default()
        };
        second.add_edits("a.ts".into(), 2);
        first.absorb(second);
        assert_eq!(first.edits[&PathBuf::from("a.ts")], 3);
        assert_eq!(first.moves.len(), 1);
        assert_eq!(first.notes, vec!["n".to_string()]);
    }
}
